use crate::utils::*;

pub fn run() {
    let url = "https://sapo.salta.world/pricing";
    
    print_info("Opening pricing page...");
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()
            .ok();
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .ok();
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .ok();
    }
}
