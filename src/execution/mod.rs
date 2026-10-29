#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunState { Idle, Running, Completed, Failed, Stopped }

#[derive(Clone, Debug)]
pub struct ExecutionState {
    pub state: RunState,
    pub output: String,
    pub error: String,
    pub exit_code: Option<i32>,
    #[cfg(not(target_arch = "wasm32"))]
    child: Option<std::process::Child>,
    #[cfg(not(target_arch = "wasm32"))]
    rx: Option<std::sync::mpsc::Receiver<StreamMessage>>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
enum StreamMessage { Stdout(String), Stderr(String) }

impl Default for ExecutionState {
    fn default() -> Self {
        Self {
            state: RunState::Idle,
            output: String::new(),
            error: String::new(),
            exit_code: None,
            #[cfg(not(target_arch = "wasm32"))]
            child: None,
            #[cfg(not(target_arch = "wasm32"))]
            rx: None,
        }
    }
}

impl ExecutionState {
    pub fn clear(&mut self) {
        self.output.clear(); self.error.clear(); self.exit_code = None;
        #[cfg(not(target_arch = "wasm32"))]
        { self.child = None; self.rx = None; }
        self.state = RunState::Idle;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_native(&mut self, path: &str, source: &str) {
        use std::{fs, io::Read, process::{Command, Stdio}, sync::mpsc, thread};
        self.stop();
        self.state = RunState::Running;
        self.output.clear(); self.error.clear(); self.exit_code = None;

        let extension = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
        let mut temp_binary = None;
        let mut command = match extension.as_str() {
            "py" => { let mut c = Command::new("python"); c.arg("-c").arg(source); c }
            "js" | "mjs" => { let mut c = Command::new("node"); c.arg("-e").arg(source); c }
            "sh" => { let mut c = Command::new("sh"); c.arg("-c").arg(source); c }
            "rs" => {
                let dir = std::env::temp_dir();
                let stem = format!("vyron_{}_main", std::process::id());
                let src_path = dir.join(format!("{stem}.rs"));
                #[cfg(windows)] let bin_path = dir.join(format!("{stem}.exe"));
                #[cfg(not(windows))] let bin_path = dir.join(&stem);
                if let Err(e) = fs::write(&src_path, source) { self.error = e.to_string(); self.state = RunState::Failed; return; }
                let compile = Command::new("rustc").arg(&src_path).arg("-o").arg(&bin_path).output();
                match compile {
                    Ok(out) if out.status.success() => {
                        self.output.push_str(&String::from_utf8_lossy(&out.stdout));
                        self.error.push_str(&String::from_utf8_lossy(&out.stderr));
                        temp_binary = Some(bin_path.clone());
                        Command::new(bin_path)
                    }
                    Ok(out) => {
                        self.error.push_str(&String::from_utf8_lossy(&out.stderr));
                        let _ = fs::remove_file(src_path);
                        self.exit_code = out.status.code(); self.state = RunState::Failed; return;
                    }
                    Err(e) => { self.error = e.to_string(); self.state = RunState::Failed; return; }
                }
            }
            _ => { self.error = "No native runner configured for this file type.".into(); self.state = RunState::Failed; return; }
        };

        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        match command.spawn() {
            Ok(mut child) => {
                let (tx, rx) = mpsc::channel();
                if let Some(mut stdout) = child.stdout.take() {
                    let tx2 = tx.clone(); thread::spawn(move || { let mut buf=String::new(); let _=stdout.read_to_string(&mut buf); let _=tx2.send(StreamMessage::Stdout(buf)); });
                }
                if let Some(mut stderr) = child.stderr.take() {
                    thread::spawn(move || { let mut buf=String::new(); let _=stderr.read_to_string(&mut buf); let _=tx.send(StreamMessage::Stderr(buf)); });
                }
                self.child = Some(child);
                self.rx = Some(rx);
                if let Some(path) = temp_binary { std::thread::spawn(move || { std::thread::sleep(std::time::Duration::from_secs(30)); let _=std::fs::remove_file(path); }); }
            }
            Err(e) => { self.error = e.to_string(); self.state = RunState::Failed; }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll(&mut self) {
        if let Some(rx) = &self.rx {
            while let Ok(message) = rx.try_recv() {
                match message { StreamMessage::Stdout(s) => self.output.push_str(&s), StreamMessage::Stderr(s) => self.error.push_str(&s) }
            }
        }
        if let Some(child) = &mut self.child {
            match child.try_wait() {
                Ok(Some(status)) => { self.exit_code = status.code(); self.state = if status.success() { RunState::Completed } else { RunState::Failed }; self.child = None; self.rx = None; }
                Ok(None) => {}
                Err(e) => { self.error.push_str(&format!("\nProcess wait error: {e}")); self.state = RunState::Failed; self.child = None; self.rx = None; }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn poll(&mut self) {}

    #[cfg(target_arch = "wasm32")]
    pub fn run_web_preview(&mut self, path: &str, source: &str) {
        self.state = RunState::Completed;
        self.error.clear();
        self.exit_code = Some(0);
        self.output.clear();
        let ext = path.rsplit('.').next().unwrap_or("");
        match ext {
            "json" => match serde_json::from_str::<serde_json::Value>(source) {
                Ok(value) => self.output = format!("VYRON JSON check: valid\nTop-level type: {}\n", if value.is_object(){"object"}else if value.is_array(){"array"}else{"scalar"}),
                Err(e) => { self.error = format!("JSON error: {e}"); self.state = RunState::Failed; self.exit_code = Some(1); }
            },
            "md" => self.output = format!("VYRON Markdown preview check\nCharacters: {}\nLines: {}\n", source.chars().count(), source.lines().count()),
            _ => self.output = format!("VYRON Web Runner\n\nThe browser sandbox cannot spawn an operating-system compiler or terminal process.\nFile: {path}\nLanguage: {}\n\nUse the native VYRON build for real process execution, or connect a remote runner in a future release.\n", crate::editor::language_for(path)),
        }
    }

    pub fn stop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
            self.state = RunState::Stopped;
            self.output.push_str("\n[process stopped by user]\n");
            self.rx = None;
            return;
        }
        #[cfg(target_arch = "wasm32")]
        { self.state = RunState::Stopped; self.output.push_str("\n[web run cancelled]\n"); }
    }
}
