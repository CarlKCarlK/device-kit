//! Background button monitoring with a spawned task.
//!
//! See the [`button_watch!`](crate::button_watch!) macro for usage.

use embassy_futures::select::{Either, select};
use embassy_rp::Peri;
use embassy_rp::gpio::{Input, Pin, Pull};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};

use super::{BUTTON_DEBOUNCE_DELAY, LONG_PRESS_DURATION, PressDuration, PressedTo};

// ============================================================================
// ButtonWatchStatic - Static resources for button monitoring
// ============================================================================

/// Static resources for button monitoring created by [`button_watch!`](crate::button_watch!).
///
/// See the [`button_watch!`](crate::button_watch!) macro for usage.
pub struct ButtonWatchStatic {
    signal: Signal<CriticalSectionRawMutex, PressDuration>,
}

impl ButtonWatchStatic {
    /// Creates static resources for button monitoring.
    ///
    /// This is automatically called by the [`button_watch!`](crate::button_watch!) macro.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            signal: Signal::new(),
        }
    }

    /// Returns a reference to the internal signal.
    #[must_use]
    pub const fn signal(&self) -> &Signal<CriticalSectionRawMutex, PressDuration> {
        &self.signal
    }
}

// ============================================================================
// ButtonWatch - Handle for background button monitoring
// ============================================================================

/// A device abstraction for background button monitoring.
///
/// Created by the [`button_watch!`](crate::button_watch!) macro. Unlike [`Button`](super::Button), button
/// detection runs in a dedicated background task, so press events are never missed
/// even when the main task is busy (e.g., updating displays or processing long operations).
///
/// See the [`button_watch!`](crate::button_watch!) macro for usage examples.
pub struct ButtonWatch {
    signal: &'static Signal<CriticalSectionRawMutex, PressDuration>,
}

impl ButtonWatch {
    /// Creates a new `ButtonWatch` from static resources.
    ///
    /// This is automatically called by the [`button_watch!`](crate::button_watch!) macro.
    #[must_use]
    pub fn new(button_watch_static: &'static ButtonWatchStatic) -> Self {
        Self {
            signal: button_watch_static.signal(),
        }
    }

    /// Waits for the next button press event.
    ///
    /// Returns whether the press was [`PressDuration::Short`] or [`PressDuration::Long`].
    ///
    /// This method never misses presses because detection happens in a dedicated
    /// background task. Safe to use in `select()` branches without starvation.
    pub async fn wait_for_press_duration(&self) -> PressDuration {
        self.signal.wait().await
    }
}

// ============================================================================
// Background task implementation
// ============================================================================

/// Background task that monitors button state and fires events.
///
/// Never call directly - spawned automatically by the [`button_watch!`](crate::button_watch!) macro.
#[doc(hidden)]
pub async fn button_watch_task<P: Pin>(
    pin: Peri<'static, P>,
    pressed_to: PressedTo,
    signal: &'static Signal<CriticalSectionRawMutex, PressDuration>,
) -> ! {
    let pull = match pressed_to {
        PressedTo::Voltage => Pull::Down,
        PressedTo::Ground => Pull::Up,
    };
    let mut input = Input::new(pin, pull);

    loop {
        // Wait for button to be released (if pressed)
        while is_pressed(&input, pressed_to) {
            Timer::after(Duration::from_millis(1)).await;
        }
        Timer::after(BUTTON_DEBOUNCE_DELAY).await;
        while is_pressed(&input, pressed_to) {
            Timer::after(Duration::from_millis(1)).await;
        }

        // Wait for button press (debounced)
        while !is_pressed(&input, pressed_to) {
            Timer::after(Duration::from_millis(1)).await;
        }
        Timer::after(BUTTON_DEBOUNCE_DELAY).await;
        if !is_pressed(&input, pressed_to) {
            continue; // was bounce
        }

        // Measure press duration
        let press_duration = match select(
            wait_for_release(&mut input, pressed_to),
            Timer::after(LONG_PRESS_DURATION),
        )
        .await
        {
            Either::First(_) => PressDuration::Short,
            Either::Second(()) => PressDuration::Long,
        };

        signal.signal(press_duration);
    }
}

fn is_pressed(input: &Input<'static>, pressed_to: PressedTo) -> bool {
    match pressed_to {
        PressedTo::Voltage => input.is_high(),
        PressedTo::Ground => input.is_low(),
    }
}

async fn wait_for_release(input: &mut Input<'static>, pressed_to: PressedTo) {
    loop {
        if !is_pressed(input, pressed_to) {
            Timer::after(BUTTON_DEBOUNCE_DELAY).await;
            if !is_pressed(input, pressed_to) {
                break;
            }
        }
        Timer::after(Duration::from_millis(1)).await;
    }
}

/// Background task that monitors button state from an existing Input.
///
/// This variant is used when converting from a `Button` via `from_button()`.
/// Never call directly - spawned automatically by the [`button_watch!`](crate::button_watch!) macro.
#[doc(hidden)]
pub async fn button_watch_task_from_input(
    mut input: Input<'static>,
    pressed_to: PressedTo,
    signal: &'static Signal<CriticalSectionRawMutex, PressDuration>,
) -> ! {
    loop {
        // Wait for button to be released (if pressed)
        while is_pressed(&input, pressed_to) {
            Timer::after(Duration::from_millis(1)).await;
        }
        Timer::after(BUTTON_DEBOUNCE_DELAY).await;
        while is_pressed(&input, pressed_to) {
            Timer::after(Duration::from_millis(1)).await;
        }

        // Wait for button press (debounced)
        while !is_pressed(&input, pressed_to) {
            Timer::after(Duration::from_millis(1)).await;
        }
        Timer::after(BUTTON_DEBOUNCE_DELAY).await;
        if !is_pressed(&input, pressed_to) {
            continue; // was bounce
        }

        // Measure press duration
        let press_duration = match select(
            wait_for_release(&mut input, pressed_to),
            Timer::after(LONG_PRESS_DURATION),
        )
        .await
        {
            Either::First(_) => PressDuration::Short,
            Either::Second(()) => PressDuration::Long,
        };

        signal.signal(press_duration);
    }
}

// ============================================================================
// button_watch! macro
// ============================================================================

/// Creates a button monitoring device with a background task.
///
/// Unlike [`Button`](super::Button), which requires polling via `wait_for_press_duration()`,
/// this macro creates a button monitor that runs in a dedicated background task. Press events
/// are detected continuously and can be retrieved via `wait_for_press_duration()` without
/// risk of being missed due to busy main task operations.
///
/// # Use Cases
///
/// Use `button_watch!` instead of [`Button`](super::Button) when:
/// - The main task performs long operations (display updates, network calls, etc.)
/// - Using `select()` with other futures that might starve button detection
/// - Running time-critical operations where button presses must never be missed
///
///  # Parameters
///
/// - `name`: The struct name for the button watch device
/// - `pin`: The GPIO pin connected to the button
///
/// Optional:
/// - `vis`: Visibility modifier (default: private)
///
/// # Example
///
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// use device_kit::button_watch;
/// use device_kit::button::PressDuration;
/// use device_kit::button::PressedTo;
/// use embassy_executor::Spawner;
/// # #[panic_handler]
/// # fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }
///
/// button_watch! {
///     ResetButton {
///         pin: PIN_13,
///     }
/// }
///
/// async fn example(p: embassy_rp::Peripherals, spawner: Spawner) {
///     // Create the button monitor (spawns background task automatically)
///     let reset_button = ResetButton::new(p.PIN_13, PressedTo::Ground, spawner)
///         .expect("Failed to create button monitor");
///
///     loop {
///         // Wait for button press - never misses events even if this loop is slow
///         match reset_button.wait_for_press_duration().await {
///             PressDuration::Short => {
///                 // Handle short press
/// #               break;
///             }
///             PressDuration::Long => {
///                 // Handle long press
/// #               break;
///             }
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! button_watch {
    // Entry point with optional visibility
    (
        $(#[$meta:meta])*
        $vis:vis $name:ident {
            pin: $pin:ident,
        }
    ) => {
        $crate::__button_watch_impl! {
            meta: [$(#[$meta])*],
            vis: $vis,
            name: $name,
            pin: $pin
        }
    };

    // Entry point with default (private) visibility
    (
        $(#[$meta:meta])*
        $name:ident {
            pin: $pin:ident,
        }
    ) => {
        $crate::__button_watch_impl! {
            meta: [$(#[$meta])*],
            vis: ,
            name: $name,
            pin: $pin
        }
    };
}

/// Implementation macro for `button_watch!`.
///
/// Do not call directly - use [`button_watch!`](crate::button_watch!) instead.
#[doc(hidden)]
#[macro_export]
macro_rules! __button_watch_impl {
    (
        meta: [$(#[$meta:meta])*],
        vis: $vis:vis,
        name: $name:ident,
        pin: $pin:ident
    ) => {
        ::paste::paste! {
            $(#[$meta])*
            #[doc = concat!(
                "Button monitor generated by [`button_watch!`].\n\n",
                "Monitors button presses in a background task. ",
                "See the [button_watch module documentation](mod@$crate::button::button_watch) for usage."
            )]
            $vis struct $name {
                button_watch: $crate::button::button_watch::ButtonWatch,
            }

            impl $name {
                /// Creates a new button monitor and spawns its background task.
                ///
                /// # Parameters
                ///
                /// - `pin`: GPIO pin for the button
                /// - `pressed_to`: How the button is wired ([`PressedTo::Ground`] or [`PressedTo::Voltage`])
                /// - `spawner`: Task spawner for background operations
                ///
                /// # Errors
                ///
                /// Returns an error if the background task cannot be spawned.
                pub fn new(
                    pin: impl Into<::embassy_rp::Peri<'static, ::embassy_rp::peripherals::$pin>>,
                    pressed_to: $crate::button::PressedTo,
                    spawner: ::embassy_executor::Spawner,
                ) -> $crate::Result<&'static Self> {
                    static BUTTON_WATCH_STATIC: $crate::button::button_watch::ButtonWatchStatic =
                        $crate::button::button_watch::ButtonWatchStatic::new();
                    static BUTTON_WATCH_CELL: ::static_cell::StaticCell<$name> =
                        ::static_cell::StaticCell::new();

                    let pin = pin.into();
                    let task_token = [<$name:snake _task>](
                        pin,
                        pressed_to,
                        BUTTON_WATCH_STATIC.signal(),
                    );
                    spawner.spawn(task_token).map_err($crate::Error::TaskSpawn)?;

                    let button_watch = $crate::button::button_watch::ButtonWatch::new(
                        &BUTTON_WATCH_STATIC,
                    );

                    let instance = BUTTON_WATCH_CELL.init($name { button_watch });
                    Ok(instance)
                }

                /// Creates a button monitor from an existing `Button` and spawns its background task.
                ///
                /// This is useful for converting a `Button` returned from `WifiAuto::connect()`
                /// into a `ButtonWatch` for background monitoring.
                ///
                /// # Parameters
                ///
                /// - `button`: An existing button (e.g., from `WifiAuto::connect()`)
                /// - `spawner`: Task spawner for background operations
                ///
                /// # Errors
                ///
                /// Returns an error if the background task cannot be spawned.
                ///
                /// # Example
                ///
                /// ```rust,no_run
                /// # #![no_std]
                /// # #![no_main]
                /// # use device_kit::button_watch;
                /// # use embassy_executor::Spawner;
                /// # #[panic_handler]
                /// # fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }
                /// button_watch! {
                ///     ResetButton {
                ///         pin: PIN_13,
                ///     }
                /// }
                ///
                /// async fn example(
                ///     button: device_kit::button::Button<'static>,
                ///     spawner: Spawner,
                /// ) -> device_kit::Result<()> {
                ///     // Convert Button from WifiAuto into ButtonWatch
                ///     let reset_button = ResetButton::from_button(button, spawner)?;
                ///
                ///     // Now button monitoring happens in background
                ///     loop {
                ///         let press = reset_button.wait_for_press_duration().await;
                ///         // Handle press...
                /// #       break;
                ///     }
                /// #   Ok(())
                /// }
                /// ```
                pub fn from_button(
                    button: $crate::button::Button<'static>,
                    spawner: ::embassy_executor::Spawner,
                ) -> $crate::Result<&'static Self> {
                    static BUTTON_WATCH_STATIC: $crate::button::button_watch::ButtonWatchStatic =
                        $crate::button::button_watch::ButtonWatchStatic::new();
                    static BUTTON_WATCH_CELL: ::static_cell::StaticCell<$name> =
                        ::static_cell::StaticCell::new();

                    let (input, pressed_to) = button.into_parts();
                    let task_token = [<$name:snake _task_from_input>](
                        input,
                        pressed_to,
                        BUTTON_WATCH_STATIC.signal(),
                    );
                    spawner.spawn(task_token).map_err($crate::Error::TaskSpawn)?;

                    let button_watch = $crate::button::button_watch::ButtonWatch::new(
                        &BUTTON_WATCH_STATIC,
                    );

                    let instance = BUTTON_WATCH_CELL.init($name { button_watch });
                    Ok(instance)
                }
            }

            impl ::core::ops::Deref for $name {
                type Target = $crate::button::button_watch::ButtonWatch;

                fn deref(&self) -> &Self::Target {
                    &self.button_watch
                }
            }

            #[::embassy_executor::task]
            async fn [<$name:snake _task>](
                pin: ::embassy_rp::Peri<'static, ::embassy_rp::peripherals::$pin>,
                pressed_to: $crate::button::PressedTo,
                signal: &'static ::embassy_sync::signal::Signal<
                    ::embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
                    $crate::button::PressDuration
                >,
            ) -> ! {
                $crate::button::button_watch::button_watch_task(pin, pressed_to, signal).await
            }

            #[::embassy_executor::task]
            async fn [<$name:snake _task_from_input>](
                input: ::embassy_rp::gpio::Input<'static>,
                pressed_to: $crate::button::PressedTo,
                signal: &'static ::embassy_sync::signal::Signal<
                    ::embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
                    $crate::button::PressDuration
                >,
            ) -> ! {
                $crate::button::button_watch::button_watch_task_from_input(input, pressed_to, signal).await
            }
        }
    };
}
