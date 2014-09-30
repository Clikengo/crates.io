use std::collections::HashMap;

use conduit::{mod, Handler, Request};
use semver;

use cargo_registry::db::RequestTransaction;
use cargo_registry::krate::Crate;
use cargo_registry::version::{EncodableVersion, Version};

#[deriving(Decodable)]
struct VersionList { versions: Vec<EncodableVersion> }
#[deriving(Decodable)]
struct VersionResponse { version: EncodableVersion }

fn sv(s: &str) -> semver::Version {
    semver::Version::parse(s).unwrap()
}

#[test]
fn index() {
    let (_b, app, middle) = ::app();
    let mut req = ::req(app, conduit::Get, "/versions");
    let mut response = ok_resp!(middle.call(&mut req));
    let json: VersionList = ::json(&mut response);
    assert_eq!(json.versions.len(), 0);

    let (v1, v2) = {
        let req = &mut req as &mut Request;
        let tx = req.tx().unwrap();
        let c = Crate::find_or_insert(tx, "foo", 32).unwrap();
        let v1 = Version::insert(tx, c.id, &sv("1.0.0"), &HashMap::new()).unwrap();
        let v2 = Version::insert(tx, c.id, &sv("1.0.1"), &HashMap::new()).unwrap();
        (v1, v2)
    };
    req.with_query(format!("ids[]={}&ids[]={}", v1.id, v2.id));
    let mut response = ok_resp!(middle.call(&mut req));
    let json: VersionList = ::json(&mut response);
    assert_eq!(json.versions.len(), 2);
}

#[test]
fn show() {
    let (_b, app, middle) = ::app();
    let mut req = ::req(app, conduit::Get, "/versions");
    let v = {
        let req = &mut req as &mut Request;
        let tx = req.tx().unwrap();
        let krate = Crate::find_or_insert(tx, "foo", 32).unwrap();
        Version::insert(tx, krate.id, &sv("1.0.0"), &HashMap::new()).unwrap()
    };
    req.with_path(format!("/versions/{}", v.id).as_slice());
    let mut response = ok_resp!(middle.call(&mut req));
    let json: VersionResponse = ::json(&mut response);
    assert_eq!(json.version.id, v.id);
}
