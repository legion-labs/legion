use hassle_rs::DxcIncludeHandler;
use lgn_embedded_fs::EMBEDDED_FS;

pub struct FileServerIncludeHandler; // stack

impl DxcIncludeHandler for FileServerIncludeHandler {
    fn load_source(&mut self, file_name: String) -> Option<String> {
        // The compiler append "./" to the file name, we need to remove it
        let fixed_up_path = if let Some(pos) = file_name.find("crate://") {
            &file_name[pos..]
        } else {
            &file_name[..]
        };
        EMBEDDED_FS.read_to_string(fixed_up_path).ok()
    }
}
