- **`KeyPolicy::Explicit` overlays take the keys only on request.** The new policy is
  key-capable like `OnDemand`, but a click never makes the overlay key and showing alone never
  requests focus; only `OverlayHandle::request_keys` admits it. `release_keys`, a hide, or the
  macOS panel resigning the key gives them back, and a click still cannot take them again. Use
  it for a mouse-first panel that enters a keyboard mode only on an explicit gesture. `OnDemand`
  keeps click-to-key and `None` stays never-key.
