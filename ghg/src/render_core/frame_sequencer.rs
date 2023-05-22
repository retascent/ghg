use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

use crate::utils::prelude::*;

pub trait FrameParams = Clone;

/// This acts as the single-frame context. When this object is destroyed, it
/// marks that the current task has reached the end of its frame, so it can be
/// queued for the next one.
///
/// It provides automatic dereferencing to this frame's parameter values
#[must_use]
pub struct FrameContext<T: FrameParams> {
	current_params: T,
	shared_params: Rc<RefCell<Option<T>>>,
}

impl<T: FrameParams> FrameContext<T> {
	fn new(shared_params: Rc<RefCell<Option<T>>>) -> Self {
		let cell: &RefCell<Option<T>> = shared_params.borrow();
		let current_value = cell.borrow().clone();
		let current_params =
			current_value.expect("Unable to create ParamContext with empty params");
		Self { current_params, shared_params }
	}
}

impl<T: FrameParams> Deref for FrameContext<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target { &self.current_params }
}

impl<T: FrameParams> Drop for FrameContext<T> {
	fn drop(&mut self) {
		let cell: &RefCell<Option<T>> = self.shared_params.borrow();
		assert!(cell.borrow().is_some());
		cell.replace(None);
	}
}

pub struct FrameSequencer<T: FrameParams> {
	running_gates: RefCell<HashMap<usize, (Rc<RefCell<Option<T>>>, Option<Waker>)>>,
	next_id: Cell<usize>,
}

impl<T: FrameParams> FrameSequencer<T> {
	pub fn new() -> Self {
		Self { running_gates: RefCell::new(HashMap::default()), next_id: Cell::new(0) }
	}

	fn register(self: &Rc<FrameSequencer<T>>) -> (usize, Rc<RefCell<Option<T>>>) {
		let next_id = self.next_id.get();
		self.next_id.replace(next_id + 1);

		self.running_gates.borrow_mut().insert(next_id, (Rc::new(RefCell::new(None)), None));
		(next_id, self.running_gates.borrow().get(&next_id).unwrap().0.clone())
	}

	fn mark_all_running(self: &Rc<FrameSequencer<T>>, params: T) {
		for (_id, (running, maybe_waker)) in self.running_gates.borrow().iter() {
			let running_params: &RefCell<Option<T>> = running.borrow();
			running_params.replace(Some(params.clone()));

			if maybe_waker.is_some() {
				maybe_waker.as_ref().unwrap().clone().wake();
			}
		}
	}

	fn update_waker(self: &Rc<FrameSequencer<T>>, gate_id: usize, waker: Waker) {
		let mut gates = self.running_gates.borrow_mut();
		let entry =
			gates.get_mut(&gate_id).expect(format!("Could not find gate id {gate_id}").as_str());

		(*entry).1 = Some(waker);
	}

	fn remove_gate(self: &Rc<FrameSequencer<T>>, gate_id: usize) {
		let mut gates = self.running_gates.borrow_mut();
		gates.remove(&gate_id).expect(format!("Unable to find gate ID {gate_id}").as_str());
		ghg_log!("Removed gate ID {gate_id}");
	}
}

pub struct FrameGate<T: FrameParams> {
	sequencer: Rc<FrameSequencer<T>>,
	id: usize,
	pub params: Rc<RefCell<Option<T>>>,
	pub name: String,
	frame_waker: Cell<Option<Waker>>,
}

impl<T: FrameParams> FrameGate<T> {
	pub fn new(sequencer: Rc<FrameSequencer<T>>, name: String) -> Self {
		let (id, params) = sequencer.register();
		Self { sequencer, id, params, name, frame_waker: Cell::new(None) }
	}
}

impl<T: FrameParams> Future for &FrameGate<T> {
	type Output = FrameContext<T>;

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		self.as_mut().frame_waker.replace(Some(cx.waker().clone()));
		self.sequencer.update_waker(self.id, cx.waker().clone());

		let running_params: &RefCell<Option<T>> = self.params.borrow();
		if running_params.borrow().is_some() {
			// let params = running_params.replace(None);
			// assert!(params.is_some());
			Poll::Ready(FrameContext::new(self.params.clone()))
		} else {
			Poll::Pending
		}
	}
}

impl<T: FrameParams> Drop for FrameGate<T> {
	fn drop(&mut self) {
		ghg_log!("Removing gate {} (id={})", self.name, self.id);
		self.sequencer.remove_gate(self.id);
	}
}

pub struct FrameMarker<T: FrameParams> {
	sequencer: Rc<FrameSequencer<T>>,
}

impl<T: FrameParams> FrameMarker<T> {
	pub fn new(sequencer: Rc<FrameSequencer<T>>) -> Self { Self { sequencer } }

	pub fn frame(&self, params: T) { self.sequencer.mark_all_running(params) }
}
