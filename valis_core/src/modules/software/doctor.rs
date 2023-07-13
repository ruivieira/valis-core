use crate::modules::software::definitions::{Component, Installation};

pub fn doctor(components: Vec<Component>) {
    for component in components {
        component.check_install();
    }
}
