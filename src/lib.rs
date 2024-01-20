//! left-right-cell is a lockfree, eventually consistent cell created using the `left-right` crate.
//! It allows readers to read from the cell without ever blocking while the writer might block when writing.
//! This is achived by storing to copies of the data one for the readers and one for the writer.
#![deny(missing_docs)]
use std::ops::Deref;

use left_right::Absorb;

struct SetOp<T>(T);

impl<T> Absorb<SetOp<T>> for Inner<T>
where
    T: Clone,
{
    fn absorb_first(&mut self, operation: &mut SetOp<T>, _: &Self) {
        self.0 = operation.0.clone();
    }

    fn absorb_second(&mut self, operation: SetOp<T>, _: &Self) {
        self.0 = operation.0;
    }

    fn drop_first(self: Box<Self>) {}

    fn sync_with(&mut self, first: &Self) {
        self.0 = first.0.clone()
    }
}

#[derive(Clone)]
struct Inner<T>(T);

/// A handle to the read half of the cell. Getting a value from the read handle will never block.
pub struct ReadHandle<T>(left_right::ReadHandle<Inner<T>>);
impl<T> ReadHandle<T> {
    /// Gets the value from the cell. Returns [`None`] if the [`WriteHandle`] as been dropped.
    pub fn get(&self) -> Option<ReadGuard<T>> {
        self.0.enter().map(|guard| ReadGuard(guard))
    }

    /// # Safety
    /// The user of this function must be sure that the [`WriteHandle`] has not been dropped.
    pub unsafe fn get_unchecked(&self) -> ReadGuard<T> {
        self.0
            .enter()
            .map(|guard| ReadGuard(guard))
            .unwrap_unchecked()
    }
}

/// A reference guard to the read half of the cell. [`WriteHandle::publish`] will block until this is dropped.
pub struct ReadGuard<'a, T>(left_right::ReadGuard<'a, Inner<T>>);

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.as_ref().0
    }
}

impl<T> Clone for ReadHandle<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        ReadHandle(self.0.clone())
    }
}

/// A handle to the write half of the cell.
/// When this handle is dropped the backing data is also dropped.
pub struct WriteHandle<T: Clone>(left_right::WriteHandle<Inner<T>, SetOp<T>>);

impl<T> WriteHandle<T>
where
    T: Clone,
{
    /// Set the value of the cell.
    pub fn set(&mut self, value: T) {
        self.0.append(SetOp(value));
    }

    /// Make the changes the to cell since the last set visible to the readers.
    pub fn publish(&mut self) {
        self.0.publish();
    }
}

/// Creates a new left-right-cell and returns the read and write handle.
pub fn new<T: Clone>(value: T) -> (WriteHandle<T>, ReadHandle<T>) {
    let (w, r) = left_right::new_from_empty::<Inner<T>, SetOp<T>>(Inner(value));
    (WriteHandle(w), ReadHandle(r))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let (mut w, r) = super::new(false);

        let t = std::thread::spawn(move || loop {
            let value = r.get().unwrap();
            if *value {
                break;
            }
        });

        w.set(true);
        w.publish();
        t.join().unwrap();
        assert!(true);
    }
}
