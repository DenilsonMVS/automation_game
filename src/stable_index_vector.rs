use std::{alloc::{self, Layout}, ops::{Add, AddAssign, Index, IndexMut, Sub, SubAssign}, ptr};
use std::marker::PhantomData;

use num_traits::{FromPrimitive, PrimInt, Unsigned, Zero};

pub trait StableId: PrimInt + Unsigned + Add + AddAssign + Sub + SubAssign + FromPrimitive {}
impl<T: PrimInt + Unsigned + Add + AddAssign + Sub + SubAssign + FromPrimitive> StableId for T {}

#[derive(Debug)]
pub struct StableIndexVector<T: Unpin, I: StableId = usize> {
    ptr: *mut u8,
    len: I,
    capacity: I,
    _marker: PhantomData<T>,
}

impl<T: Unpin, I: StableId> StableIndexVector<T, I> {
    pub fn new() -> Self {
        Self {
            ptr: ptr::null_mut(),
            len: Zero::zero(),
            capacity: Zero::zero(),
            _marker: PhantomData,
        }
    }

    unsafe fn get_layout(capacity: usize) -> (Layout, usize, usize, usize) {
        if capacity == 0 {
            return (Layout::from_size_align(0, 8).unwrap(), 0, 0, 0);
        }
        let data_layout = Layout::array::<T>(capacity).unwrap();
        let index_layout = Layout::array::<I>(capacity).unwrap();
        let rev_layout = Layout::array::<I>(capacity).unwrap();

        let (layout, index_offset) = data_layout.extend(index_layout).unwrap();
        let (layout, rev_offset) = layout.extend(rev_layout).unwrap();
        
        (layout.pad_to_align(), 0, index_offset, rev_offset)
    }

    #[inline] unsafe fn data_ptr(&self) -> *mut T {
        self.ptr as *mut T
    }

    #[inline]
    unsafe fn index_ptr(&self) -> *mut I {
        unsafe {
            let (_, _, index_off, _) = Self::get_layout(self.capacity.to_usize().unwrap());
            self.ptr.add(index_off) as *mut I
        }
    }

    #[inline]
    unsafe fn rev_ptr(&self) -> *mut I {
        unsafe {
            let (_, _, _, rev_off) = Self::get_layout(self.capacity.to_usize().unwrap());
            self.ptr.add(rev_off) as *mut I
        }
    }

    pub fn is_valid(&self, id: I) -> bool {
        let id_u = id.to_usize().unwrap();
        if id >= self.capacity { return false; }
        unsafe {
            let pos = *self.index_ptr().add(id_u);
            pos < self.len
        }
    }

    pub fn push(&mut self, value: T) -> I {
        if self.len == self.capacity {
            self.grow();
        }

        unsafe {
            let pos = self.len.to_usize().unwrap();
            let id = *self.rev_ptr().add(pos);
            ptr::write(self.data_ptr().add(pos), value);
            
            self.len += I::one();
            id
        }
    }

    pub fn remove(&mut self, id: I) -> Option<T> {
        if !self.is_valid(id) { return None; }

        unsafe {
            let id_u = id.to_usize().unwrap();
            let pos_u = (*self.index_ptr().add(id_u)).to_usize().unwrap();
            let last_pos_i = self.len - I::one();
            let last_pos_u = last_pos_i.to_usize().unwrap();
            
            let result = ptr::read(self.data_ptr().add(pos_u));

            if pos_u != last_pos_u {
                ptr::copy_nonoverlapping(self.data_ptr().add(last_pos_u), self.data_ptr().add(pos_u), 1);
            }

            let last_id = *self.rev_ptr().add(last_pos_u);
            let last_id_u = last_id.to_usize().unwrap();

            *self.rev_ptr().add(pos_u) = last_id;
            *self.rev_ptr().add(last_pos_u) = id;

            *self.index_ptr().add(last_id_u) = I::from_usize(pos_u).unwrap();
            *self.index_ptr().add(id_u) = I::from_usize(last_pos_u).unwrap();

            self.len = last_pos_i;
            Some(result)
        }
    }

    fn grow(&mut self) {
        let old_cap_u = self.capacity.to_usize().unwrap();
        let new_cap_u = if old_cap_u == 0 { 1 } else { old_cap_u * 2 };
        let new_cap = I::from_usize(new_cap_u).unwrap();
        
        unsafe {
            let (new_layout, _, new_idx_off, new_rev_off) = Self::get_layout(new_cap_u);
            let new_ptr = alloc::alloc(new_layout);
            if new_ptr.is_null() { alloc::handle_alloc_error(new_layout); }

            let new_idx_ptr = new_ptr.add(new_idx_off) as *mut I;
            let new_rev_ptr = new_ptr.add(new_rev_off) as *mut I;

            ptr::copy_nonoverlapping(self.data_ptr(), new_ptr as *mut T, self.len.to_usize().unwrap());
            
            ptr::copy_nonoverlapping(self.index_ptr(), new_idx_ptr, old_cap_u);
            for i in old_cap_u..new_cap_u {
                let val = I::from_usize(i).unwrap();
                *new_idx_ptr.add(i) = val;
            }

            ptr::copy_nonoverlapping(self.rev_ptr(), new_rev_ptr, old_cap_u);
            for i in old_cap_u..new_cap_u {
                let val = I::from_usize(i).unwrap();
                *new_rev_ptr.add(i) = val;
            }

            let (old_layout, _, _, _) = Self::get_layout(old_cap_u);
            alloc::dealloc(self.ptr, old_layout);

            

            self.ptr = new_ptr;
            self.capacity = new_cap;
        }
    }

    pub fn iter_enumerate_unordered(&self) -> impl Iterator<Item = (I, &T)> {
        let rev_base = unsafe { self.rev_ptr() };
        let data_base = unsafe { self.data_ptr() };
        
        (0..self.len.to_usize().unwrap()).map(move |pos| unsafe {
            (*rev_base.add(pos), &*data_base.add(pos))
        })
    }

    pub fn iter_enumerate_unordered_mut(&mut self) -> impl Iterator<Item = (I, &mut T)> {
        let rev_base = unsafe { self.rev_ptr() };
        let data_base = unsafe { self.data_ptr() };
        
        (0..self.len.to_usize().unwrap()).map(move |pos| unsafe {
            (*rev_base.add(pos), &mut *data_base.add(pos))
        })
    }

    pub fn iter_unordered(&self) -> impl Iterator<Item = &T> {
        let data_base = unsafe { self.data_ptr() };
        
        (0..self.len.to_usize().unwrap()).map(move |pos| unsafe {
            &*data_base.add(pos)
        })
    }

    pub fn iter_unordered_mut(&mut self) -> impl Iterator<Item = &mut T> {
        let data_base = unsafe { self.data_ptr() };
        
        (0..self.len.to_usize().unwrap()).map(move |pos| unsafe {
            &mut *data_base.add(pos)
        })
    }

    pub fn get(&self, id: I) -> Option<&T> {
        if !self.is_valid(id) {
            None
        } else {
            unsafe {
                Some(&*self.data_ptr().add((*self.index_ptr().add(id.to_usize().unwrap())).to_usize().unwrap()))
            }
        }
    }

    pub fn get_mut(&mut self, id: I) -> Option<&mut T> {
        if !self.is_valid(id) {
            None
        } else {
            unsafe {
                Some(&mut *self.data_ptr().add((*self.index_ptr().add(id.to_usize().unwrap())).to_usize().unwrap()))
            }
        }
    }
}

impl<T: Unpin> Index<usize> for StableIndexVector<T> {
    type Output = T;
    fn index(&self, id: usize) -> &Self::Output {
        self.get(id).unwrap()
    }
}

impl<T: Unpin> IndexMut<usize> for StableIndexVector<T> {
    fn index_mut(&mut self, id: usize) -> &mut Self::Output {
        self.get_mut(id).unwrap()
    }
}

impl<T: Unpin, I: StableId> Drop for StableIndexVector<T, I> {
    fn drop(&mut self) {
        if !self.capacity.is_zero() {
            unsafe {
                for i in 0..self.len.to_usize().unwrap() {
                    ptr::drop_in_place(self.data_ptr().add(i));
                }
                let (layout, _, _, _) = Self::get_layout(self.capacity.to_usize().unwrap());
                alloc::dealloc(self.ptr, layout);
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

    use crate::stable_index_vector::StableIndexVector;

    #[test]
    fn test_basico_push_get() {
        let mut v = StableIndexVector::new();
        let id0 = v.push(10);
        let id1 = v.push(20);

        assert_eq!(v[id0], 10);
        assert_eq!(v[id1], 20);
        assert_eq!(v.is_valid(id0), true);
        assert_eq!(v.is_valid(id1), true);
    }

    #[test]
    fn test_reciclagem_de_id() {
        let mut v = StableIndexVector::new();
        let id0 = v.push("A");
        let id1 = v.push("B");

        // Remove o primeiro elemento
        v.remove(id0);
        assert!(!v.is_valid(id0));

        // O próximo push DEVE reciclar o ID que foi liberado
        let id_novo = v.push("C");
        assert_eq!(id_novo, id0); 
        assert_eq!(v[id_novo], "C");
        assert_eq!(v[id1], "B");
    }

    #[test]
    fn test_estabilidade_apos_remocao() {
        let mut v = StableIndexVector::new();
        let _id0 = v.push(1.1);
        let id1 = v.push(2.2);
        let id2 = v.push(3.3);

        // Remove o ID 1 (o do meio). Isso causa um swap interno do ID 2.
        v.remove(id1);

        // O ID 2 deve continuar funcionando normalmente, mesmo tendo mudado de lugar físico
        assert_eq!(v[id2], 3.3);
        assert!(!v.is_valid(id1));
    }

    #[test]
    fn test_mutabilidade() {
        let mut v = StableIndexVector::new();
        let id = v.push(10);
        
        v[id] = 50; // IndexMut
        assert_eq!(v[id], 50);

        for val in v.iter_unordered_mut() {
            *val += 10;
        }
        assert_eq!(v[id], 60);
    }

    #[test]
    #[should_panic]
    fn test_panic_acesso_invalido() {
        let mut v = StableIndexVector::new();
        let id = v.push(1);
        v.remove(id);
        let _ = v[id];
    }

    #[test]
    fn test_crescimento_da_capacidade() {
        let mut v = StableIndexVector::new();
        // Força vários grow() (capacidade dobra: 1, 2, 4, 8, 16...)
        for i in 0..20 {
            v.push(i);
        }
        assert_eq!(v[19], 19);
        assert!(v.is_valid(0));
    }

    struct DropTracker(Arc<AtomicUsize>);
    impl Drop for DropTracker {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_estresse_de_crescimento_extremo() {
        let mut v = StableIndexVector::new();
        let total = 10_000;
        let mut ids = Vec::new();

        // Insere 10k elementos para forçar múltiplos reallocs (grow)
        for i in 0..total {
            ids.push(v.push(i));
        }

        assert_eq!(v.len, total);

        // Verifica se todos os dados sobreviveram às realocações
        for (i, &id) in ids.iter().enumerate() {
            assert_eq!(v[id], i);
        }
    }

    #[test]
    fn test_reciclagem_lifo_profunda() {
        let mut v = StableIndexVector::<_, u8>::new();
        
        // Pushes iniciais
        let id0 = v.push(0); // 0
        let id1 = v.push(1); // 1
        let id2 = v.push(2); // 2

        // Remove em ordem inversa
        v.remove(id2);
        v.remove(id1);
        v.remove(id0);

        // Como a reciclagem usa a posição `last_pos` no swap,
        // a ordem de reciclagem tende a ser LIFO (Last-In, First-Out)
        // para os IDs removidos.
        assert_eq!(v.push(100), id0);
        assert_eq!(v.push(200), id1);
        assert_eq!(v.push(300), id2);
    }

    #[test]
    fn test_verificacao_de_vazamento_de_memoria() {
        let drop_count = Arc::new(AtomicUsize::new(0));
        
        {
            let mut v = StableIndexVector::<_, usize>::new();
            
            for _ in 0..100 {
                v.push(DropTracker(drop_count.clone()));
            }

            // Remove 50 elementos (deve dropar 50)
            for i in 0..50 {
                v.remove(i);
            }
            assert_eq!(drop_count.load(Ordering::SeqCst), 50);

            // Força um grow com os 50 restantes
            for _ in 0..100 {
                v.push(DropTracker(drop_count.clone()));
            }
            
            // Ao final deste bloco, o vetor morre.
            // Devem ter sido dropados 50 (removidos) + 150 (restantes no vetor) = 200
        }

        assert_eq!(drop_count.load(Ordering::SeqCst), 200);
    }

    #[test]
    fn test_intercalamento_caotico() {
        let mut v = StableIndexVector::new();
        let mut ids = Vec::new();

        // 1. Enche um pouco
        for i in 0..50 { ids.push(v.push(i)); }

        // 2. Remove os pares
        for i in (0..50).step_by(2) {
            v.remove(ids[i]);
        }

        // 3. Insere novos (devem ocupar os buracos dos pares)
        for i in 0..25 {
            let id = v.push(i + 1000);
            // Verifica se o ID reciclado era realmente um dos pares removidos
            assert!(id % 2 == 0); 
        }

        // 4. Verifica integridade dos ímpares que sobraram
        for i in (1..50).step_by(2) {
            assert_eq!(v[ids[i]], i);
        }
    }

    #[test]
    fn test_iter_unordered_match() {
        let mut v = StableIndexVector::new();
        v.push(10);
        v.push(20);
        let id2 = v.push(30);
        v.remove(id2);
        v.push(40);

        // O iterador unordered deve retornar exatamente o que o Index retorna
        for (id, val) in v.iter_enumerate_unordered() {
            assert_eq!(*val, v[id]);
        }
    }

    #[test]
    fn test_zero_capacity_remove() {
        let mut v: StableIndexVector<i32> = StableIndexVector::new();
        assert_eq!(v.remove(0), None);
        assert_eq!(v.is_valid(0), false);
    }

    #[test]
    fn test_single_element_lifecycle() {
        let mut v = StableIndexVector::new();
        let id = v.push(42);
        assert_eq!(v.remove(id), Some(42));
        assert_eq!(v.len, 0);
        let id2 = v.push(84);
        assert_eq!(id, id2); // Reciclagem imediata
        assert_eq!(v[id2], 84);
    }
}
