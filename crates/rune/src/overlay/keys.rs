//! Keyboard focus for an overlay on a window of its own (X11, Windows), and what the macOS panel
//! and those windows report as `nonactivating_keys`, kept cross-platform so it compiles and runs
//! tests on every host. As with the macOS panel and a layer-shell surface, `KeyPolicy::OnDemand`
//! lets the window take the keys when `OverlayHandle::request_keys` asks or the user clicks it,
//! and showing alone never does; `KeyPolicy::Explicit` takes them only on `request_keys`, and a
//! click never does; `KeyPolicy::None` never takes them.

use crate::{KeyPolicy, OverlaySupport};

/// Whether an overlay window with `policy` may take keyboard focus. Windows makes one that may
/// without `WS_EX_NOACTIVATE`; X11 gives it the focus on `request_keys` and on a click.
#[cfg_attr(not(any(target_os = "windows", all(unix, not(target_os = "ios"), not(target_os = "android")))), allow(dead_code))]
pub(crate) fn can_take_keys(policy: KeyPolicy) -> bool {
    policy.takes_keys()
}

/// Whether a click on an overlay with `policy` takes the keys by itself. `Explicit` and `None`
/// never do; an explicit overlay takes them only through `OverlayHandle::request_keys`.
#[cfg_attr(not(any(target_os = "windows", target_os = "macos", all(unix, not(target_os = "ios"), not(target_os = "android")))), allow(dead_code))]
pub(crate) fn click_takes_keys(policy: KeyPolicy) -> bool {
    matches!(policy, KeyPolicy::OnDemand | KeyPolicy::Exclusive)
}

/// What an overlay with `policy` reports as `OverlayCapabilities::nonactivating_keys`.
/// `focuser`: the platform piece that focuses it is there (X11's own connection, which a server
/// without the SHAPE extension doesn't get; always there for the macOS panel and on Windows).
#[cfg_attr(not(any(target_os = "windows", all(unix, not(target_os = "ios"), not(target_os = "android")))), allow(dead_code))]
pub(crate) fn keys_support(policy: KeyPolicy, focuser: bool) -> OverlaySupport {
    if can_take_keys(policy) && focuser { OverlaySupport::Supported } else { OverlaySupport::Unsupported }
}

/// The macOS panel's key admission for `policy`, kept pure so every host tests the policy.
/// `OnDemand` and `Exclusive` may become key at any time; `Explicit` only while a
/// `request_keys` holds, until `release` (a release, a hide, or AppKit resigning the key);
/// `None` never may. A click never admits an explicit panel on its own.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KeyGate {
    policy: KeyPolicy,
    requested: bool,
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
impl KeyGate {
    pub(crate) fn new(policy: KeyPolicy) -> Self {
        Self { policy, requested: false }
    }

    /// What the panel's `canBecomeKeyWindow` answers now.
    pub(crate) fn can_become_key(self) -> bool {
        match self.policy {
            KeyPolicy::None => false,
            KeyPolicy::OnDemand | KeyPolicy::Exclusive => true,
            KeyPolicy::Explicit => self.requested,
        }
    }

    /// `OverlayHandle::request_keys`: the next key-window call may take effect.
    pub(crate) fn request(&mut self) {
        self.requested = true;
    }

    /// `release_keys`, a hide, or AppKit resigning the key. `OnDemand` and `Exclusive` still
    /// may become key on the next click; `Explicit` must be asked again.
    pub(crate) fn release(&mut self) {
        self.requested = false;
    }

    /// Whether a click takes the keys, as [`click_takes_keys`].
    pub(crate) fn click_takes_keys(self) -> bool {
        click_takes_keys(self.policy)
    }
}

/// The gate a panel shares with its `canBecomeKeyWindow` implementation.
#[cfg(target_os = "macos")]
pub(crate) type SharedKeyGate = std::rc::Rc<std::cell::Cell<KeyGate>>;

#[cfg(test)]
#[path = "keys_tests.rs"]
mod tests;
