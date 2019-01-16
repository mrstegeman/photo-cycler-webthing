extern crate actix_web;
extern crate rand;
#[macro_use]
extern crate serde_json;
extern crate webthing;

use actix_web::{fs, App};
use rand::Rng;
use std::any::Any;
use std::env;
use std::fs::DirEntry;
use std::path::Path;
use std::process;
use std::sync::{Arc, RwLock, Weak};
use std::vec::Drain;
use std::{thread, time};
use webthing::action::Action;
use webthing::event::Event;
use webthing::property::{BaseProperty, Property, ValueForwarder};
use webthing::server::{ActionGenerator, AppState, ThingsType, WebThingServer};
use webthing::thing::{BaseThing, Thing};

struct UpdateRateForwarder(Weak<RwLock<u64>>);

impl ValueForwarder for UpdateRateForwarder {
    fn set_value(&mut self, value: serde_json::Value) -> Result<serde_json::Value, &'static str> {
        if !value.is_u64() {
            return Err("Invalid value");
        }

        let val: u64 = value.as_u64().unwrap() as u64;
        match self.0.upgrade() {
            Some(update_rate) => {
                *update_rate.write().unwrap() = val;
                Ok(value)
            }
            None => Err("Client reference disappeared"),
        }
    }
}

struct PhotoCyclerThing {
    base: BaseThing,
}

impl PhotoCyclerThing {
    fn new(photos_path: String, static_path: String) -> PhotoCyclerThing {
        let mut base = BaseThing::new(
            "Photo Cycler".to_owned(),
            Some(vec![]),
            Some("Photo Cycler".to_owned()),
        );

        let update_rate: Arc<RwLock<u64>> = Arc::new(RwLock::new(5));

        let update_rate_description = json!({
            "type": "integer",
            "description": "Photo cycle rate",
            "minimum": 0,
            "unit": "second",
            "title": "Update Rate",
        });
        let update_rate_description = update_rate_description.as_object().unwrap().clone();
        base.add_property(Box::new(BaseProperty::new(
            "updateRate".to_owned(),
            json!(5),
            Some(Box::new(UpdateRateForwarder(Arc::downgrade(&update_rate)))),
            Some(update_rate_description),
        )));

        let image_description = json!({
            "@type": "ImageProperty",
            "type": "null",
            "description": "Current image",
            "title": "Image",
            "readOnly": true,
            "links": [
                {
                    "rel": "alternate",
                    "href": "/static/current.jpg",
                    "mediaType": "image/jpeg",
                },
            ],
        });
        let image_description = image_description.as_object().unwrap().clone();
        base.add_property(Box::new(BaseProperty::new(
            "image".to_owned(),
            serde_json::Value::Null,
            None,
            Some(image_description),
        )));

        let update_rate_cloned = update_rate.clone();
        thread::spawn(move || {
            let static_path = Path::new(&static_path);
            let photos_path = Path::new(&photos_path);
            loop {
                let update_rate = update_rate_cloned.clone();
                let update_rate = *update_rate.read().unwrap();

                thread::sleep(time::Duration::from_millis(update_rate * 1000));

                let result = photos_path.read_dir();
                if result.is_err() {
                    continue;
                }
                let result = result.unwrap();

                let files: Vec<DirEntry> = result
                    .filter(|e| e.is_ok())
                    .map(|e| e.unwrap())
                    .filter(|e| e.file_type().unwrap().is_file())
                    .filter(|e| {
                        let name = e.file_name().to_str().unwrap().to_lowercase();
                        name.ends_with(".jpg") || name.ends_with(".jpeg")
                    })
                    .collect();

                if files.len() == 0 {
                    continue;
                }

                let mut rng = rand::thread_rng();
                let choice: usize = rng.gen_range(0, files.len());

                let link = static_path.join("current.jpg");
                let image = &files[choice].path();

                if link.exists() {
                    let result = std::fs::remove_file(link.clone());
                    if result.is_err() {
                        eprintln!("Failed to remove symlink: {:?}", result.unwrap_err());
                        continue;
                    }
                }

                #[allow(deprecated)]
                let result = std::fs::soft_link(image, link);
                if result.is_err() {
                    eprintln!("Failed to symlink file: {:?}", result.unwrap_err());
                }
            }
        });

        PhotoCyclerThing { base: base }
    }
}

impl Thing for PhotoCyclerThing {
    fn as_thing_description(&self) -> serde_json::Map<String, serde_json::Value> {
        self.base.as_thing_description()
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut Any {
        self
    }

    fn get_href(&self) -> String {
        self.base.get_href()
    }

    fn get_href_prefix(&self) -> String {
        self.base.get_href_prefix()
    }

    fn get_ui_href(&self) -> Option<String> {
        self.base.get_ui_href()
    }

    fn set_href_prefix(&mut self, prefix: String) {
        self.base.set_href_prefix(prefix)
    }

    fn set_ui_href(&mut self, href: String) {
        self.base.set_ui_href(href)
    }

    fn get_name(&self) -> String {
        self.base.get_name()
    }

    fn get_context(&self) -> String {
        self.base.get_context()
    }

    fn get_type(&self) -> Vec<String> {
        self.base.get_type()
    }

    fn get_description(&self) -> String {
        self.base.get_description()
    }

    fn get_property_descriptions(&self) -> serde_json::Map<String, serde_json::Value> {
        self.base.get_property_descriptions()
    }

    fn get_action_descriptions(&self, action_name: Option<String>) -> serde_json::Value {
        self.base.get_action_descriptions(action_name)
    }

    fn get_event_descriptions(&self, event_name: Option<String>) -> serde_json::Value {
        self.base.get_event_descriptions(event_name)
    }

    fn add_property(&mut self, property: Box<Property>) {
        self.base.add_property(property)
    }

    fn remove_property(&mut self, property_name: String) {
        self.base.remove_property(property_name)
    }

    fn find_property(&mut self, property_name: String) -> Option<&mut Box<Property>> {
        self.base.find_property(property_name)
    }

    fn get_property(&self, property_name: String) -> Option<serde_json::Value> {
        self.base.get_property(property_name)
    }

    fn get_properties(&self) -> serde_json::Map<String, serde_json::Value> {
        self.base.get_properties()
    }

    fn has_property(&self, property_name: String) -> bool {
        self.base.has_property(property_name)
    }

    fn get_action(
        &self,
        action_name: String,
        action_id: String,
    ) -> Option<Arc<RwLock<Box<Action>>>> {
        self.base.get_action(action_name, action_id)
    }

    fn add_event(&mut self, event: Box<Event>) {
        self.base.add_event(event)
    }

    fn add_available_event(
        &mut self,
        name: String,
        metadata: serde_json::Map<String, serde_json::Value>,
    ) {
        self.base.add_available_event(name, metadata)
    }

    fn add_action(
        &mut self,
        action: Arc<RwLock<Box<Action>>>,
        input: Option<&serde_json::Value>,
    ) -> Result<(), &str> {
        self.base.add_action(action, input)
    }

    fn remove_action(&mut self, action_name: String, action_id: String) -> bool {
        self.base.remove_action(action_name, action_id)
    }

    fn add_available_action(
        &mut self,
        name: String,
        metadata: serde_json::Map<String, serde_json::Value>,
    ) {
        self.base.add_available_action(name, metadata)
    }

    fn add_subscriber(&mut self, ws_id: String) {
        self.base.add_subscriber(ws_id)
    }

    fn remove_subscriber(&mut self, ws_id: String) {
        self.base.remove_subscriber(ws_id)
    }

    fn add_event_subscriber(&mut self, name: String, ws_id: String) {
        self.base.add_event_subscriber(name, ws_id)
    }

    fn remove_event_subscriber(&mut self, name: String, ws_id: String) {
        self.base.remove_event_subscriber(name, ws_id)
    }

    fn property_notify(&mut self, name: String, value: serde_json::Value) {
        self.base.property_notify(name, value)
    }

    fn action_notify(&mut self, action: serde_json::Map<String, serde_json::Value>) {
        self.base.action_notify(action)
    }

    fn event_notify(&mut self, name: String, event: serde_json::Map<String, serde_json::Value>) {
        self.base.event_notify(name, event)
    }

    fn start_action(&mut self, name: String, id: String) {
        self.base.start_action(name, id)
    }

    fn cancel_action(&mut self, name: String, id: String) {
        self.base.cancel_action(name, id)
    }

    fn finish_action(&mut self, name: String, id: String) {
        self.base.finish_action(name, id)
    }

    fn drain_queue(&mut self, ws_id: String) -> Vec<Drain<String>> {
        self.base.drain_queue(ws_id)
    }
}

struct Generator;

impl ActionGenerator for Generator {
    fn generate(
        &self,
        _thing: Weak<RwLock<Box<Thing>>>,
        _name: String,
        _input: Option<&serde_json::Value>,
    ) -> Option<Box<Action>> {
        None
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <photos_path> <static_path>", args[0]);
        process::exit(1);
    }

    let photos_path = Path::new(&args[1]).canonicalize();
    if photos_path.is_err() {
        eprintln!("Photos directory does not exist");
        process::exit(1);
    }
    let photos_path = photos_path.unwrap();

    let static_path = Path::new(&args[2]).canonicalize();
    if static_path.is_err() {
        eprintln!("Static directory does not exist");
        process::exit(1);
    }
    let static_path = static_path.unwrap();

    let thing: Arc<RwLock<Box<Thing + 'static>>> =
        Arc::new(RwLock::new(Box::new(PhotoCyclerThing::new(
            photos_path.to_str().unwrap().to_string(),
            static_path.to_str().unwrap().to_string(),
        ))));

    let static_path = static_path.to_str().unwrap().to_string();
    let configure = move |app: App<AppState>| {
        app.handler(
            "/static",
            fs::StaticFiles::new(&static_path.clone()).unwrap(),
        )
    };

    let mut server = WebThingServer::new(
        ThingsType::Single(thing),
        Some(8888),
        None,
        None,
        Box::new(Generator),
        Some(Box::new(configure)),
    );
    server.create();
    server.start();
}
