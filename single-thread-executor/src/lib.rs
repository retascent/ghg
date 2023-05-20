use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::Context;

use async_channel::{Receiver, Sender};
use futures::task::{waker_ref, ArcWake};

/// Because this is a single-thread executor, we don't have to be as worried
/// about Send requirements for the Futures we're executing. Here, we wrap our
/// non-Send Futures in a type which we assert is Send, which lets us use
/// Sender/Receiver for convenience.
///
/// I may be misunderstanding the actual problem with this solution, but it
/// works for now, so I'm going to stick with it until I understand Rust async
/// internals better.
struct ISwearItsFine(Pin<Box<dyn Future<Output = ()> + 'static>>);

unsafe impl Send for ISwearItsFine {}

impl ISwearItsFine {
	fn from(future: impl Future<Output = ()> + 'static) -> Self { Self(Box::pin(future)) }
}

pub struct Executor {
	ready_queue: Receiver<Arc<Task>>,
}

/// `Spawner` spawns new futures onto the task channel.
#[derive(Clone)]
pub struct Spawner {
	task_sender: Sender<Arc<Task>>,
}

/// A future that can reschedule itself to be polled by an `Executor`.
struct Task {
	/// In-progress future that should be pushed to completion.
	///
	/// The `Mutex` is not necessary for correctness, since we only have
	/// one thread executing tasks at once. However, Rust isn't smart
	/// enough to know that `future` is only mutated from one thread,
	/// so we need to use the `Mutex` to prove thread-safety. A production
	/// executor would not need this, and could use `UnsafeCell` instead.
	future: Mutex<Option<ISwearItsFine>>,

	/// Handle to place the task itself back onto the task queue.
	task_sender: Sender<Arc<Task>>,
}

pub fn new_executor_and_spawner() -> (Executor, Spawner) {
	const MAX_QUEUED_TASKS: usize = 10_000;
	let (task_sender, ready_queue) = async_channel::bounded(MAX_QUEUED_TASKS);
	(Executor { ready_queue }, Spawner { task_sender })
}

impl Spawner {
	pub fn spawn(&self, future: impl Future<Output = ()> + 'static) {
		let future = ISwearItsFine::from(future);
		let task = Arc::new(Task {
			future: Mutex::new(Some(future)),
			task_sender: self.task_sender.clone(),
		});
		self.task_sender.try_send(task).expect("too many tasks queued");
	}
}

impl ArcWake for Task {
	fn wake_by_ref(arc_self: &Arc<Self>) {
		// Implement `wake` by sending this task back onto the task channel
		// so that it will be polled again by the executor.
		let cloned = arc_self.clone();
		arc_self.task_sender.try_send(cloned).expect("too many tasks queued");
	}
}

impl Executor {
	pub async fn run(&self) {
		while let Ok(task) = self.ready_queue.recv().await {
			// Take the future, and if it has not yet completed (is still Some),
			// poll it in an attempt to complete it.
			let mut future_slot = task.future.lock().unwrap();
			if let Some(mut future) = future_slot.take() {
				// Create a `LocalWaker` from the task itself
				let waker = waker_ref(&task);
				let context = &mut Context::from_waker(&waker);
				// `BoxFuture<T>` is a type alias for
				// `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
				// We can get a `Pin<&mut dyn Future + Send + 'static>`
				// from it by calling the `Pin::as_mut` method.
				if future.0.as_mut().poll(context).is_pending() {
					// We're not done processing the future, so put it
					// back in its task to be run again in the future.
					*future_slot = Some(future);
				}
			}
		}
	}
}
