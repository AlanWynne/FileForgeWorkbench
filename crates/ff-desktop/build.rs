fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("ffwb.manifest");
        res.compile().expect("failed to compile Windows resources");
    }
}
