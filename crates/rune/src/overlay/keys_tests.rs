use super::{can_take_keys, keys_support, KeyGate};
use crate::{KeyPolicy, OverlayInput, OverlaySupport};

#[test]
fn only_a_key_capable_overlay_window_takes_keys() {
    assert!(!can_take_keys(KeyPolicy::None), "never activates, as today");
    assert!(can_take_keys(KeyPolicy::OnDemand));
    assert!(can_take_keys(KeyPolicy::Exclusive));
    assert!(can_take_keys(KeyPolicy::Explicit), "capable, but only on request");
    assert_eq!(keys_support(KeyPolicy::None, true), OverlaySupport::Unsupported);
    assert_eq!(keys_support(KeyPolicy::OnDemand, true), OverlaySupport::Supported);
    assert_eq!(keys_support(KeyPolicy::Explicit, true), OverlaySupport::Supported, "nonactivating keys stay supported");
    assert_eq!(keys_support(KeyPolicy::OnDemand, false), OverlaySupport::Unsupported, "no X11 connection to focus it with");
}

/// An explicit panel may become key only after `request_keys`; before it, a click cannot make
/// it key, and a click never asks for the keys on its own.
#[test]
fn explicit_becomes_key_only_after_request_keys() {
    let mut gate = KeyGate::new(KeyPolicy::Explicit);
    assert!(!gate.can_become_key(), "a fresh explicit panel is not key-capable yet");
    assert!(!gate.click_takes_keys(), "a click never requests keys");
    gate.request();
    assert!(gate.can_become_key(), "the explicit request enables the key window");
    gate.release();
    assert!(!gate.can_become_key(), "release gives the keys back");
    assert!(!gate.click_takes_keys(), "and a later click still cannot take them");
}

/// `OnDemand` keeps click-to-key and `None` never becomes key; `Exclusive` is `OnDemand` on
/// hosts without exclusive layer-shell interactivity.
#[test]
fn on_demand_and_exclusive_keep_click_to_key() {
    for policy in [KeyPolicy::OnDemand, KeyPolicy::Exclusive] {
        let gate = KeyGate::new(policy);
        assert!(gate.can_become_key(), "{policy:?}");
        assert!(gate.click_takes_keys(), "{policy:?}");
        let mut gate = gate;
        gate.release();
        assert!(gate.can_become_key(), "{policy:?} may take the keys again on the next click");
    }
    let none = KeyGate::new(KeyPolicy::None);
    assert!(!none.can_become_key());
    assert!(!none.click_takes_keys());
    let mut none = none;
    none.request();
    assert!(!none.can_become_key(), "None stays never-key even on request");
}

/// `KeyPolicy` means the same on every backend that can take keys: `OnDemand`, `Exclusive` and
/// `Explicit` may, `None` never does. Only XWayland, whose override-redirect window never takes
/// focus, refuses them.
#[test]
fn key_policy_agrees_across_backends() {
    use crate::overlay::xwayland::xwayland_allowed;
    use crate::platform::layer_shell_machine::keyboard::supports_keys as layer;
    for policy in [KeyPolicy::None, KeyPolicy::OnDemand, KeyPolicy::Exclusive, KeyPolicy::Explicit] {
        let wants = policy.takes_keys();
        assert_eq!(can_take_keys(policy), wants, "X11 and Windows windows, {policy:?}");
        assert_eq!(keys_support(policy, true) == OverlaySupport::Supported, wants, "macOS panel, X11 and Windows windows, {policy:?}");
        assert_eq!(layer(policy, 4, true), wants, "layer-shell, {policy:?}");
        assert_eq!(xwayland_allowed(OverlayInput::None, policy, true), !wants, "XWayland, {policy:?}");
    }
}
