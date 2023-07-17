use std::sync::{Arc, Mutex};

use httpz::cookie::Cookie;

/// TODO
///
// TODO: Can `Rc<RefCell<T>>` be used so I don't need to await a borrow and use Tokio specific API's???
// TODO: The `Mutex` will block. This isn't great, work to remove it. The Tokio `Mutex` makes everything annoyingly async so I don't use it.
#[derive(Debug, Clone)]
pub struct CookieJar(Arc<Mutex<httpz::cookie::CookieJar>>);

impl CookieJar {
    pub(super) fn new(cookies: Arc<Mutex<httpz::cookie::CookieJar>>) -> Self {
        Self(cookies)
    }

    /// Returns a reference to the `Cookie` inside this jar with the name
    /// `name`. If no such cookie exists, returns `None`.
    #[allow(clippy::panic)] // TODO: Remove this
    pub fn get(&self, name: &str) -> Option<Cookie<'static>> {
        #[allow(clippy::unwrap_used)] // TODO
        self.0.lock().unwrap().get(name).cloned() // TODO: `cloned` is cringe avoid it by removing `Mutex`?
    }

    /// Adds an "original" `cookie` to this jar. If an original cookie with the
    /// same name already exists, it is replaced with `cookie`. Cookies added
    /// with `add` take precedence and are not replaced by this method.
    ///
    /// Adding an original cookie does not affect the [delta](#method.delta)
    /// computation. This method is intended to be used to seed the cookie jar
    /// with cookies received from a client's HTTP message.
    ///
    /// For accurate `delta` computations, this method should not be called
    /// after calling `remove`.
    #[allow(clippy::panic)] // TODO: Remove this
    pub fn add_original(&self, cookie: Cookie<'static>) {
        #[allow(clippy::unwrap_used)] // TODO
        self.0.lock().unwrap().add_original(cookie)
    }

    /// Adds `cookie` to this jar. If a cookie with the same name already
    /// exists, it is replaced with `cookie`.
    #[allow(clippy::panic)] // TODO: Remove this
    pub fn add(&self, cookie: Cookie<'static>) {
        #[allow(clippy::unwrap_used)] // TODO
        self.0.lock().unwrap().add(cookie);
    }

    /// Removes `cookie` from this jar. If an _original_ cookie with the same
    /// name as `cookie` is present in the jar, a _removal_ cookie will be
    /// present in the `delta` computation. To properly generate the removal
    /// cookie, `cookie` must contain the same `path` and `domain` as the cookie
    /// that was initially set.
    ///
    /// A "removal" cookie is a cookie that has the same name as the original
    /// cookie but has an empty value, a max-age of 0, and an expiration date
    /// far in the past. See also [`Cookie::make_removal()`].
    ///
    /// Removing a new cookie does not result in a _removal_ cookie unless
    /// there's an original cookie with the same name:
    #[allow(clippy::panic)] // TODO: Remove this
    pub fn remove(&self, cookie: Cookie<'static>) {
        #[allow(clippy::unwrap_used)] // TODO
        self.0.lock().unwrap().remove(cookie)
    }

    /// Removes `cookie` from this jar completely. This method differs from
    /// `remove` in that no delta cookie is created under any condition. Neither
    /// the `delta` nor `iter` methods will return a cookie that is removed
    /// using this method.
    #[allow(clippy::panic)] // TODO: Remove this
    pub fn force_remove(&self, cookie: &Cookie<'_>) {
        #[allow(clippy::unwrap_used)] // TODO
        self.0.lock().unwrap().force_remove(cookie)
    }

    /// Removes all delta cookies, i.e. all cookies not added via
    /// [`CookieJar::add_original()`], from this `CookieJar`. This undoes any
    /// changes from [`CookieJar::add()`] and [`CookieJar::remove()`]
    /// operations.
    #[allow(clippy::panic)] // TODO: Remove this
    pub fn reset_delta(&self) {
        #[allow(clippy::unwrap_used)] // TODO
        self.0.lock().unwrap().reset_delta()
    }

    // /// Returns an iterator over cookies that represent the changes to this jar
    // /// over time. These cookies can be rendered directly as `Set-Cookie` header
    // /// values to affect the changes made to this jar on the client.
    // pub fn delta(&self) -> Delta {
    //     self.0.lock().unwrap().delta()
    // }

    // /// Returns an iterator over all of the cookies present in this jar.
    // pub fn iter(&self) -> Iter {
    //     self.0.lock().unwrap().iter()
    // }

    // /// Returns a read-only `PrivateJar` with `self` as its parent jar using the
    // /// key `key` to verify/decrypt cookies retrieved from the child jar. Any
    // /// retrievals from the child jar will be made from the parent jar.
    // #[cfg(feature = "private")]
    // #[cfg_attr(all(nightly, doc), doc(cfg(feature = "private")))]
    // pub fn private<'a>(&'a self, key: &Key) -> PrivateJar<&'a Self> {
    //     PrivateJar::new(self, key)
    // }

    // /// Returns a read/write `PrivateJar` with `self` as its parent jar using
    // /// the key `key` to sign/encrypt and verify/decrypt cookies added/retrieved
    // /// from the child jar.
    // ///
    // /// Any modifications to the child jar will be reflected on the parent jar,
    // /// and any retrievals from the child jar will be made from the parent jar.
    // #[cfg(feature = "private")]
    // #[cfg_attr(all(nightly, doc), doc(cfg(feature = "private")))]
    // pub fn private_mut<'a>(&'a mut self, key: &Key) -> PrivateJar<&'a mut Self> {
    //     PrivateJar::new(self, key)
    // }

    // /// Returns a read-only `SignedJar` with `self` as its parent jar using the
    // /// key `key` to verify cookies retrieved from the child jar. Any retrievals
    // /// from the child jar will be made from the parent jar.
    // #[cfg(feature = "signed")]
    // #[cfg_attr(all(nightly, doc), doc(cfg(feature = "signed")))]
    // pub fn signed<'a>(&'a self, key: &Key) -> SignedJar<&'a Self> {
    //     SignedJar::new(self, key)
    // }

    // /// Returns a read/write `SignedJar` with `self` as its parent jar using the
    // /// key `key` to sign/verify cookies added/retrieved from the child jar.
    // ///
    // /// Any modifications to the child jar will be reflected on the parent jar,
    // /// and any retrievals from the child jar will be made from the parent jar.
    // #[cfg(feature = "signed")]
    // #[cfg_attr(all(nightly, doc), doc(cfg(feature = "signed")))]
    // pub fn signed_mut<'a>(&'a mut self, key: &Key) -> SignedJar<&'a mut Self> {
    //     SignedJar::new(self, key)
    // }
}
