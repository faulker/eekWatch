use sysinfo::Disks;

pub fn display_disks() {
    let display_size = u64::pow(1024, 3);
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        let usage = disk.usage();
        let used = usage.total_written_bytes / display_size;
        let total = disk.total_space() / display_size;
        let free = disk.available_space() / display_size;

        print!(
            "Mount Point: {:?}
Name: {:?}
File System Type: {:?}
Drive Type: {}
Removable: {}
Size:
    - Total: {} GB
    - Used: {} GB
    - Free: {} GB
-------------------------------\n",
            disk.mount_point(),
            disk.name(),
            disk.file_system(),
            disk.kind(),
            disk.is_removable(),
            total,
            used,
            free
        );
    }
}
