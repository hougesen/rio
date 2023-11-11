// WA is a fork of https://github.com/rust-windowing/wa/
// wa is is licensed under Apache 2.0 license https://github.com/rust-windowing/wa/blob/master/LICENSE

use std::time::Duration;

use crate::{
    event::Event,
    event_loop::{EventLoop, EventLoopWindowTarget},
};

/// The return status for `pump_events`
pub enum PumpStatus {
    /// Continue running external loop.
    Continue,
    /// Exit external loop.
    Exit(i32),
}

/// Additional methods on [`EventLoop`] for pumping events within an external event loop
pub trait EventLoopExtPumpEvents {
    /// A type provided by the user that can be passed through [`Event::UserEvent`].
    type UserEvent;

    /// Pump the `EventLoop` to check for and dispatch pending events.
    ///
    /// This API is designed to enable applications to integrate wa into an
    /// external event loop, for platforms that can support this.
    ///
    /// The given `timeout` limits how long it may block waiting for new events.
    ///
    /// Passing a `timeout` of `Some(Duration::ZERO)` would ensure your external
    /// event loop is never blocked but you would likely need to consider how
    /// to throttle your own external loop.
    ///
    /// Passing a `timeout` of `None` means that it may wait indefinitely for new
    /// events before returning control back to the external loop.
    ///
    /// **Note:** This is not a portable API, and its usage involves a number of
    /// caveats and trade offs that should be considered before using this API!
    ///
    /// You almost certainly shouldn't use this API, unless you absolutely know it's
    /// the only practical option you have.
    ///
    /// ## Synchronous events
    ///
    /// Some events _must_ only be handled synchronously via the closure that
    /// is passed to wa so that the handler will also be synchronized with
    /// the window system and operating system.
    ///
    /// This is because some events are driven by a window system callback
    /// where the window systems expects the application to have handled the
    /// event before returning.
    ///
    /// **These events can not be buffered and handled outside of the closure
    /// passed to wa.**
    ///
    /// As a general rule it is not recommended to ever buffer events to handle
    /// them outside of the closure passed to wa since it's difficult to
    /// provide guarantees about which events are safe to buffer across all
    /// operating systems.
    ///
    /// Notable events that will certainly create portability problems if
    /// buffered and handled outside of wa include:
    /// - `RedrawRequested` events, used to schedule rendering.
    ///
    ///     macOS for example uses a `drawRect` callback to drive rendering
    /// within applications and expects rendering to be finished before
    /// the `drawRect` callback returns.
    ///
    ///     For portability it's strongly recommended that applications should
    /// keep their rendering inside the closure provided to wa.
    /// - Any lifecycle events, such as `Suspended` / `Resumed`.
    ///
    ///     The handling of these events needs to be synchronized with the
    /// operating system and it would never be appropriate to buffer a
    /// notification that your application has been suspended or resumed and
    /// then handled that later since there would always be a chance that
    /// other lifecycle events occur while the event is buffered.
    ///
    /// ## Supported Platforms
    /// - Windows
    /// - Linux
    /// - MacOS
    /// - Android
    ///
    /// ## Unsupported Platforms
    /// - **Web:**  This API is fundamentally incompatible with the event-based way in which
    /// Web browsers work because it's not possible to have a long-running external
    /// loop that would block the browser and there is nothing that can be
    /// polled to ask for new new events. Events are delivered via callbacks based
    /// on an event loop that is internal to the browser itself.
    /// - **iOS:** It's not possible to stop and start an `NSApplication` repeatedly on iOS so
    /// there's no way to support the same approach to polling as on MacOS.
    ///
    /// ## Platform-specific
    /// - **Windows**: The implementation will use `PeekMessage` when checking for
    ///   window messages to avoid blocking your external event loop.
    ///
    /// - **MacOS**: The implementation works in terms of stopping the global `NSApp`
    ///   whenever the application `RunLoop` indicates that it is preparing to block
    ///   and wait for new events.
    ///
    ///   This is very different to the polling APIs that are available on other
    ///   platforms (the lower level polling primitives on MacOS are private
    ///   implementation details for `NSApp` which aren't accessible to application
    ///   developers)
    ///
    ///   It's likely this will be less efficient than polling on other OSs and
    ///   it also means the `NSApp` is stopped while outside of the wa
    ///   event loop - and that's observable (for example to crates like `rfd`)
    ///   because the `NSApp` is global state.
    ///
    ///   If you render outside of wa you are likely to see window resizing artifacts
    ///   since MacOS expects applications to render synchronously during any `drawRect`
    ///   callback.
    fn pump_events<F>(
        &mut self,
        timeout: Option<Duration>,
        event_handler: F,
    ) -> PumpStatus
    where
        F: FnMut(Event<Self::UserEvent>, &EventLoopWindowTarget<Self::UserEvent>);
}

impl<T> EventLoopExtPumpEvents for EventLoop<T> {
    type UserEvent = T;

    fn pump_events<F>(
        &mut self,
        timeout: Option<Duration>,
        event_handler: F,
    ) -> PumpStatus
    where
        F: FnMut(Event<Self::UserEvent>, &EventLoopWindowTarget<Self::UserEvent>),
    {
        self.event_loop.pump_events(timeout, event_handler)
    }
}