use super::super::core;

pub struct Component {
    pub name: String,
    pub executable: String,
    pub dependencies: Option<Vec<Component>>,
    pub install_darwin: Vec<String>,
    pub install_linux: Vec<String>,
}

pub trait Installation {
    fn install(&self);
    fn check_install(&self);
}

impl Installation for Component {
    fn install(&self) {
        let os = core::get_os();
        if os == "macos" {
            println!("🍏 Installing for {}", core::get_os());
            if self.dependencies.is_some() {
                println!("🧰 Installing dependencies");
                let dependencies = self.dependencies.as_ref().unwrap();
                for dependency in dependencies {
                    // install(dependency);
                    dependency.install();
                }
            }
            println!("🧰 Installing {}", &self.name);
            for command in &self.install_darwin {
                println!("\t⚙️ {}", command);
                core::run(&command);
            }
        }
    }
    fn check_install(&self) {
        if core::in_path(self.executable.as_str()) {
            println!("✅ {} is installed", self.name);
        } else {
            println!("❌ {} is not installed", self.name);
        }
    }
}
