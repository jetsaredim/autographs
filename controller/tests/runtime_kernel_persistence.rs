use std::{fs, path::PathBuf};

#[test]
fn deploy_role_disables_core_and_kernel_dump_persistence() {
    let deploy_tasks = read_repo("deploy/ansible/roles/autographs_deploy/tasks/main.yml");
    let kernel_tasks =
        read_repo("deploy/ansible/roles/autographs_deploy/tasks/kernel_persistence.yml");
    let coredump_config =
        read_repo("deploy/ansible/roles/autographs_deploy/templates/autographs-coredump.conf.j2");
    let controller_quadlet = read_repo(
        "deploy/ansible/roles/autographs_deploy/templates/autographs-controller.container.j2",
    );

    assert!(deploy_tasks.contains("ansible.builtin.include_tasks: kernel_persistence.yml"));
    assert_eq!(controller_quadlet.matches("LimitCORE=0").count(), 1);
    assert!(coredump_config.contains("[Coredump]\nStorage=none\nProcessSizeMax=0"));

    assert!(kernel_tasks.contains("name: kdump.service"));
    assert!(kernel_tasks.contains("state: stopped"));
    assert!(kernel_tasks.contains("enabled: false"));
    assert!(kernel_tasks.contains("masked: true"));
    assert!(!kernel_tasks.contains("failed_when: false"));
    assert!(!kernel_tasks.contains("ignore_errors:"));

    assert!(kernel_tasks.contains("- --info=ALL"));
    assert!(kernel_tasks.contains("- --update-kernel=ALL"));
    assert!(kernel_tasks.contains("- --remove-args=crashkernel"));
    assert!(
        kernel_tasks.contains("when: \"'crashkernel' in autographs_deploy_grubby_info.stdout\"")
    );
    assert!(kernel_tasks.contains("Report required operator-approved reboot"));
    assert!(kernel_tasks.contains("reboot is required before"));
    assert!(!kernel_tasks.contains("ansible.builtin.reboot"));
    assert!(!kernel_tasks.lines().any(|line| {
        let line = line.trim();
        line == "- reboot" || line.starts_with("cmd: reboot")
    }));
}

#[test]
fn kernel_persistence_slice_does_not_mutate_swap_or_oled_storage() {
    let kernel_tasks =
        read_repo("deploy/ansible/roles/autographs_deploy/tasks/kernel_persistence.yml");

    for forbidden in [
        "swapoff",
        "swapon",
        "mkswap",
        "/var/oled",
        "lvremove",
        "lvreduce",
        "lvextend",
        "xfs_growfs",
        "ansible.posix.mount",
        "umount",
    ] {
        assert!(
            !kernel_tasks.contains(forbidden),
            "C4 core/Kdump slice must not contain storage mutation token {forbidden}"
        );
    }
}

#[test]
fn ci_runs_the_runtime_kernel_persistence_contract() {
    let ci = read_repo(".github/workflows/ci.yml");
    let validation = read_repo("scripts/validate-runtime.sh");

    assert!(ci.contains("bash scripts/validate-runtime.sh"));
    assert!(validation.contains(
        "cargo test --manifest-path controller/Cargo.toml --test runtime_kernel_persistence"
    ));
    for required_path in [
        "deploy/ansible/roles/autographs_deploy/tasks/kernel_persistence.yml",
        "deploy/ansible/roles/autographs_deploy/templates/autographs-coredump.conf.j2",
        "deploy/ansible/roles/autographs_deploy/templates/autographs-controller.container.j2",
        "controller/tests/runtime_kernel_persistence.rs",
    ] {
        assert!(
            validation.contains(required_path),
            "runtime validation must reference {required_path}"
        );
    }
}

fn read_repo(relative: &str) -> String {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("controller parent")
        .to_path_buf();
    fs::read_to_string(repo.join(relative)).expect("read repository artifact")
}
