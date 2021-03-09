use std::num::NonZeroUsize;

/// A generic threadpool implementation
pub trait ThreadPool: Send {
    /// Spawn `func` as a task into this threadpool
    fn spawn<F>(&self, func: F)
    where
        F: FnOnce() + Send + 'static;

    /// Returns the number of threads in this threadpool
    fn max_threads(&self) -> NonZeroUsize;
}

/// A [`ThreadPool`] implementation using the rayon global threadpool
#[cfg(feature = "rayon")]
#[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
#[derive(Debug)]
pub struct RayonThreadPool;

#[cfg(feature = "rayon")]
impl ThreadPool for RayonThreadPool {
    #[inline]
    fn spawn<F>(&self, func: F)
    where
        F: FnOnce() + Send + 'static,
    {
        rayon_core::spawn_fifo(func)
    }

    fn max_threads(&self) -> NonZeroUsize {
        NonZeroUsize::new(rayon_core::current_num_threads())
            .unwrap_or_else(|| NonZeroUsize::new(1).unwrap())
    }
}
