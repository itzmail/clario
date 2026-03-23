use crate::models::app_info::AppInfo;
use crate::models::file_info::FileInfo;

pub enum ScanEvent {
    Progress(String),        // Mengirim nama folder yang sedang dipindai
    Finished(Vec<FileInfo>), // Mengirim hasil akhir seluruh pindaian
    FinishedApps(Vec<AppInfo>), // Mengirim hasil akhir Apps
}
