use std::{fs, path::PathBuf};

#[test]
fn caddy_static_routes_serve_admin_and_current_static_release() {
    let caddyfile = read_repo("deploy/ansible/roles/autographs_deploy/files/Caddyfile");
    let caddy_quadlet =
        read_repo("deploy/ansible/roles/autographs_deploy/templates/autographs-caddy.container.j2");
    let deploy_tasks = read_repo("deploy/ansible/roles/autographs_deploy/tasks/main.yml");

    assert!(caddyfile.contains("@operator path /api/operator /api/operator/*"));
    assert!(caddyfile.contains("respond @operator 404"));
    assert!(caddyfile.contains("handle /admin/api/*"));
    assert!(caddyfile.contains("reverse_proxy autographs-controller:8080"));
    assert!(caddyfile.contains("handle_path /admin/*"));
    assert!(caddyfile.contains("root * /srv/autographs/admin"));
    assert!(caddyfile.contains("http://:8081"));
    assert!(caddyfile.contains("root * /srv/autographs/static/current"));
    assert!(caddyfile.contains("file_server"));
    assert!(!caddyfile.contains("reverse_proxy autographs-app:3000"));

    assert!(caddy_quadlet.contains("Volume=autographs-static.volume:/srv/autographs/static:ro"));
    assert!(!caddy_quadlet.contains("autographs-app.service"));
    assert!(
        caddy_quadlet
            .contains("PublishPort=127.0.0.1:{{ autographs_deploy_candidate_preview_port }}:8081")
    );
    assert!(
        !caddy_quadlet.contains("PublishPort={{ autographs_deploy_candidate_preview_port }}:8081")
    );

    assert!(deploy_tasks.contains("Require promoted static release before Caddy cutover"));
    assert!(deploy_tasks.contains("current/manifest.json"));
    assert!(deploy_tasks.contains("Stop and disable retired Next.js app service"));
    assert!(deploy_tasks.contains("Remove retired Next.js app quadlet"));
    assert!(deploy_tasks.contains("Remove retired Next.js app container"));
    assert!(!deploy_tasks.contains("src: autographs-app.container.j2"));
    assert!(!deploy_tasks.contains("autographs_app_image"));
    assert!(
        deploy_tasks.contains(
            "http://127.0.0.1:{{ autographs_deploy_candidate_preview_port }}/manifest.json"
        )
    );
}

#[test]
fn controller_quadlet_keeps_private_api_off_host_ports() {
    let controller_quadlet = read_repo(
        "deploy/ansible/roles/autographs_deploy/templates/autographs-controller.container.j2",
    );

    assert!(controller_quadlet.contains("Network=autographs.network"));
    assert!(
        controller_quadlet.contains("Volume=autographs-static.volume:/var/lib/autographs/static")
    );
    assert!(!controller_quadlet.contains("PublishPort="));
}

#[test]
fn deploy_tasks_hash_rotation_discards_preserved_plaintext_credentials() {
    let deploy_tasks = read_repo("deploy/ansible/roles/autographs_deploy/tasks/main.yml");
    let select_start = deploy_tasks
        .find("- name: Select deployed admin credentials")
        .expect("select deployed admin credentials exists");
    let validate_start = deploy_tasks
        .find("- name: Validate deployed admin authentication secret")
        .expect("credential validation exists");
    let select_tasks = &deploy_tasks[select_start..validate_start];

    for expected in [
        "if autographs_deploy_admin_password_input | length > 0\n          and autographs_deploy_admin_password_hash_input | length == 0",
        "''\n          if autographs_deploy_admin_password_hash_input | length > 0",
        "''\n          if autographs_deploy_admin_password_input | length > 0",
    ] {
        assert!(
            select_tasks.contains(expected),
            "deploy tasks should prefer the configured hash and drop preserved plaintext with {expected}"
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
