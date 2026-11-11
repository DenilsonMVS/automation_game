use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::marker::PhantomData;
use std::ptr::NonNull;

pub unsafe trait SoaSchema {
    type Item;
    const MAX_ALIGN: usize;

    fn layout_for(capacity: usize) -> Layout;
    fn field_offset(field_idx: usize, capacity: usize) -> usize;
    unsafe fn drop_elements(base_ptr: NonNull<u8>, len: usize, capacity: usize);
    unsafe fn copy_all(
        old_ptr: NonNull<u8>,
        new_ptr: NonNull<u8>,
        len: usize,
        old_cap: usize,
        new_cap: usize,
    );
}

pub struct SoaVec<S: SoaSchema> {
    ptr: NonNull<u8>,
    len: usize,
    capacity: usize,
    _marker: PhantomData<S>,
}

impl<S: SoaSchema> SoaVec<S> {
    pub const fn new() -> Self {
        let dangling_ptr = S::MAX_ALIGN as *mut u8;
        Self {
            // SAFETY: MAX_ALIGN é sempre >= 1.
            ptr: unsafe { NonNull::new_unchecked(dangling_ptr) },
            len: 0,
            capacity: 0,
            _marker: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut vec = Self::new();
        if capacity > 0 {
            vec.reserve_exact(capacity);
        }
        vec
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub unsafe fn raw_field_ptr<T>(&self, field_idx: usize) -> *mut T {
        let offset = S::field_offset(field_idx, self.capacity);
        unsafe { self.ptr.as_ptr().add(offset) as *mut T }
    }

    #[inline(always)]
    pub unsafe fn field_slice<T>(&self, field_idx: usize) -> &[T] {
        if self.len == 0 {
            &[]
        } else {
            unsafe {
                let ptr = self.raw_field_ptr::<T>(field_idx);
                std::slice::from_raw_parts(ptr, self.len)
            }
        }
    }

    #[inline(always)]
    pub unsafe fn field_slice_mut<T>(&mut self, field_idx: usize) -> &mut [T] {
        if self.len == 0 {
            &mut []
        } else {
            unsafe {
                let ptr = self.raw_field_ptr::<T>(field_idx);
                std::slice::from_raw_parts_mut(ptr, self.len)
            }
        }
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let required = self.len + additional;
        if required <= self.capacity {
            return;
        }

        let new_layout = S::layout_for(required);
        unsafe {
            let new_alloc = alloc(new_layout);
            let new_ptr = match NonNull::new(new_alloc) {
                Some(p) => p,
                None => handle_alloc_error(new_layout),
            };

            if self.capacity > 0 && self.len > 0 {
                S::copy_all(self.ptr, new_ptr, self.len, self.capacity, required);
                let old_layout = S::layout_for(self.capacity);
                dealloc(self.ptr.as_ptr(), old_layout);
            }

            self.ptr = new_ptr;
            self.capacity = required;
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let required = self.len + additional;
        if required <= self.capacity {
            return;
        }

        let new_cap = (self.capacity * 2).max(1).max(required);
        let new_layout = S::layout_for(new_cap);

        unsafe {
            let new_alloc = alloc(new_layout);
            let new_ptr = match NonNull::new(new_alloc) {
                Some(p) => p,
                None => handle_alloc_error(new_layout),
            };

            if self.capacity > 0 && self.len > 0 {
                S::copy_all(self.ptr, new_ptr, self.len, self.capacity, new_cap);
                let old_layout = S::layout_for(self.capacity);
                dealloc(self.ptr.as_ptr(), old_layout);
            }

            self.ptr = new_ptr;
            self.capacity = new_cap;
        }
    }

    #[inline(always)]
    pub fn inc_len(&mut self) {
        self.len += 1;
    }
}

impl<S: SoaSchema> Drop for SoaVec<S> {
    fn drop(&mut self) {
        if self.capacity > 0 {
            unsafe {
                S::drop_elements(self.ptr, self.len, self.capacity);
                dealloc(self.ptr.as_ptr(), S::layout_for(self.capacity));
            }
        }
    }
}

unsafe impl<S: SoaSchema> Send for SoaVec<S> where S::Item: Send {}
unsafe impl<S: SoaSchema> Sync for SoaVec<S> where S::Item: Sync {}

macro_rules! define_soa {
    (
        $(#[$meta:meta])*
        $vis:vis struct $item:ident => struct $soa_vec:ident {
            $(
                $(#[$fmeta:meta])*
                $field:ident $(mut $field_mut:ident)? : $ftype:ty
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $soa_vec {
            raw: $crate::struct_of_arrays::SoaVec<$item>,
        }

        unsafe impl $crate::struct_of_arrays::SoaSchema for $item {
            type Item = $item;

            const MAX_ALIGN: usize = {
                let mut max_a = 1;
                $(
                    let a = std::mem::align_of::<$ftype>();
                    if a > max_a {
                        max_a = a;
                    }
                )+
                max_a
            };

            fn layout_for(capacity: usize) -> std::alloc::Layout {
                if capacity == 0 {
                    return std::alloc::Layout::from_size_align(0, Self::MAX_ALIGN).unwrap();
                }
                let mut layout = std::alloc::Layout::from_size_align(0, 1).unwrap();
                $(
                    let (next, _) = layout
                        .extend(std::alloc::Layout::array::<$ftype>(capacity).unwrap())
                        .unwrap();
                    layout = next;
                )+
                layout.pad_to_align()
            }

            fn field_offset(field_idx: usize, capacity: usize) -> usize {
                let mut layout = std::alloc::Layout::from_size_align(0, 1).unwrap();
                let mut cur = 0;
                $(
                    let (next, offset) = layout
                        .extend(std::alloc::Layout::array::<$ftype>(capacity).unwrap())
                        .unwrap();
                    if cur == field_idx {
                        return offset;
                    }
                    layout = next;
                    cur += 1;
                )+
                let _ = (layout, cur);
                panic!("Índice de campo inválido: {}", field_idx);
            }

            unsafe fn drop_elements(base_ptr: std::ptr::NonNull<u8>, len: usize, capacity: usize) {
                if std::mem::needs_drop::<$item>() {
                    let mut cur = 0;
                    $(
                        let offset = Self::field_offset(cur, capacity);
                        unsafe {
                            let ptr = base_ptr.as_ptr().add(offset) as *mut $ftype;
                            for i in 0..len {
                                std::ptr::drop_in_place(ptr.add(i));
                            }
                        }
                        cur += 1;
                    )+
                    let _ = cur;
                }
            }

            unsafe fn copy_all(
                old_ptr: std::ptr::NonNull<u8>,
                new_ptr: std::ptr::NonNull<u8>,
                len: usize,
                old_cap: usize,
                new_cap: usize,
            ) {
                let mut cur = 0;
                $(
                    let old_off = Self::field_offset(cur, old_cap);
                    let new_off = Self::field_offset(cur, new_cap);
                    unsafe {
                        let src = old_ptr.as_ptr().add(old_off) as *const $ftype;
                        let dst = new_ptr.as_ptr().add(new_off) as *mut $ftype;
                        std::ptr::copy_nonoverlapping(src, dst, len);
                    }
                    cur += 1;
                )+
                let _ = cur;
            }
        }

        impl $soa_vec {
            pub const fn new() -> Self {
                Self { raw: $crate::struct_of_arrays::SoaVec::new() }
            }

            pub fn with_capacity(capacity: usize) -> Self {
                Self { raw: $crate::struct_of_arrays::SoaVec::with_capacity(capacity) }
            }

            #[inline(always)]
            pub fn len(&self) -> usize { self.raw.len() }
            #[inline(always)]
            pub fn capacity(&self) -> usize { self.raw.capacity() }
            #[inline(always)]
            pub fn is_empty(&self) -> bool { self.raw.is_empty() }

            pub fn push(&mut self, item: $item) {
                if self.raw.len() == self.raw.capacity() {
                    self.raw.reserve(1);
                }
                let idx = self.raw.len();
                let mut cur_field = 0;
                unsafe {
                    $(
                        let ptr = self.raw.raw_field_ptr::<$ftype>(cur_field);
                        std::ptr::write(ptr.add(idx), item.$field);
                        cur_field += 1;
                    )+
                    let _ = cur_field;
                    self.raw.inc_len();
                }
            }

            soa_accessors_internal!(0, $( $field $(mut $field_mut)? : $ftype ),+);
        }
    };
}

macro_rules! soa_accessors_internal {
    ($idx:expr, $field:ident $(mut $field_mut:ident)? : $ftype:ty) => {
        pub fn $field(&self) -> &[$ftype] {
            unsafe { self.raw.field_slice($idx) }
        }

        $(
            pub fn $field_mut(&mut self) -> &mut [$ftype] {
                unsafe { self.raw.field_slice_mut($idx) }
            }
        )?
    };
    ($idx:expr, $field:ident $(mut $field_mut:ident)? : $ftype:ty, $($rest_field:ident $(mut $rest_field_mut:ident)? : $rest_type:ty),+) => {
        pub fn $field(&self) -> &[$ftype] {
            unsafe { self.raw.field_slice($idx) }
        }

        $(
            pub fn $field_mut(&mut self) -> &mut [$ftype] {
                unsafe { self.raw.field_slice_mut($idx) }
            }
        )?

        soa_accessors_internal!($idx + 1, $($rest_field $(mut $rest_field_mut)? : $rest_type),+);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(crate) struct Point {
        pub(crate) x: f32,
        pub(crate) y: f32,
    }

    define_soa! {
        pub(crate) struct Point => struct Points {
            x mut x_mut: f32,
            y mut y_mut: f32,
        }
    }

    #[test]
    fn test_empty_initialization() {
        let points = Points::new();
        assert_eq!(points.len(), 0);
        assert_eq!(points.capacity(), 0);
        assert!(points.is_empty());
        assert_eq!(points.x(), &[]);
        assert_eq!(points.y(), &[]);
    }

    #[test]
    fn test_push_and_realloc_growth() {
        let mut points = Points::with_capacity(1);
        assert_eq!(points.capacity(), 1);

        points.push(Point { x: 1.0, y: 10.0 });
        assert_eq!(points.len(), 1);

        points.push(Point { x: 2.0, y: 20.0 });
        points.push(Point { x: 3.0, y: 30.0 });

        assert_eq!(points.len(), 3);
        assert!(points.capacity() >= 3);
        assert_eq!(points.x(), &[1.0, 2.0, 3.0]);
        assert_eq!(points.y(), &[10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_slice_mutation_in_place() {
        let mut points = Points::new();
        points.push(Point { x: 1.5, y: 2.5 });
        points.push(Point { x: 3.5, y: 4.5 });

        points.x_mut()[0] = 99.0;
        points.y_mut()[1] = 88.0;

        assert_eq!(points.x(), &[99.0, 3.5]);
        assert_eq!(points.y(), &[2.5, 88.0]);
    }

    #[derive(Debug, PartialEq)]
    pub(crate) struct MixedAlign {
        pub(crate) flag: u8,
        pub(crate) val_u64: u64,
        pub(crate) short: u16,
    }

    define_soa! {
        pub(crate) struct MixedAlign => struct MixedAligns {
            flag: u8,
            val_u64: u64,
            short: u16,
        }
    }

    #[test]
    fn test_mixed_alignments_and_offsets() {
        let mut items = MixedAligns::new();

        for i in 0..100 {
            items.push(MixedAlign {
                flag: (i % 255) as u8,
                val_u64: i as u64 * 1000,
                short: (i % 1000) as u16,
            });
        }

        assert_eq!(items.len(), 100);

        for i in 0..100 {
            assert_eq!(items.flag()[i], (i % 255) as u8);
            assert_eq!(items.val_u64()[i], i as u64 * 1000);
            assert_eq!(items.short()[i], (i % 1000) as u16);
        }

        let u64_ptr = items.val_u64().as_ptr() as usize;
        assert_eq!(u64_ptr % std::mem::align_of::<u64>(), 0);
    }

    pub(crate) struct DropTracker {
        pub(crate) _id: usize,
        pub(crate) counter: Arc<AtomicUsize>,
    }

    impl Drop for DropTracker {
        fn drop(&mut self) {
            self.counter.fetch_add(1, Ordering::SeqCst);
        }
    }

    pub(crate) struct ItemWithDrop {
        pub(crate) tag: u32,
        pub(crate) tracker: DropTracker,
    }

    define_soa! {
        pub(crate) struct ItemWithDrop => struct ItemWithDrops {
            tag: u32,
            tracker: DropTracker,
        }
    }

    #[test]
    fn test_drop_execution() {
        let counter = Arc::new(AtomicUsize::new(0));

        {
            let mut list = ItemWithDrops::new();
            for i in 0..10 {
                list.push(ItemWithDrop {
                    tag: i as u32,
                    tracker: DropTracker {
                        _id: i,
                        counter: Arc::clone(&counter),
                    },
                });
            }
            assert_eq!(list.len(), 10);
            assert_eq!(counter.load(Ordering::SeqCst), 0);
        }

        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_memory_layout_size() {
        assert_eq!(
            std::mem::size_of::<Points>(),
            std::mem::size_of::<usize>() * 3
        );
    }
}