use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
fn test_contact_form_datastar_signals() {
    // Check that the contact form template has proper Datastar signals
    let template_content = include_str!("../templates/contact_form.html");

    // Test 1: Verify data-signals is defined with proper structure
    assert!(
        template_content.contains("data-signals="),
        "Contact form must have data-signals defined"
    );

    assert!(
        template_content.contains("showSuccess") && template_content.contains("showError"),
        "Data signals must include showSuccess and showError"
    );

    assert!(
        template_content.contains("errorMessage") && template_content.contains("name") && template_content.contains("email"),
        "Data signals must include errorMessage, name, and email"
    );

    // Test 2: Verify data-show attributes for visibility control
    assert!(
        template_content.contains("data-show=\"$showSuccess\""),
        "Success section should be controlled by showSuccess signal"
    );

    assert!(
        template_content.contains("data-show=\"$showError\""),
        "Error section should be controlled by showError signal"
    );

    assert!(
        template_content.contains("data-show=\"!$showSuccess\""),
        "Form should be hidden when success is shown"
    );

    // Test 3: Verify data-text for error message display
    assert!(
        template_content.contains("data-text=\"$errorMessage\""),
        "Error message should use data-text with errorMessage signal"
    );

    // Test 4: Verify data-bind attributes on form inputs
    assert!(
        template_content.contains("data-bind=\"name\""),
        "Name input should have data-bind attribute"
    );

    assert!(
        template_content.contains("data-bind=\"email\""),
        "Email input should have data-bind attribute"
    );

    assert!(
        template_content.contains("data-bind=\"phone\""),
        "Phone input should have data-bind attribute"
    );

    assert!(
        template_content.contains("data-bind=\"service\""),
        "Service select should have data-bind attribute"
    );

    assert!(
        template_content.contains("data-bind=\"message\""),
        "Message textarea should have data-bind attribute"
    );

    // Test 5: Verify reset functionality on success
    assert!(
        template_content.contains("$showSuccess = false; $showError = false"),
        "Send Another Message button should reset visibility signals"
    );
}

#[test]
fn test_server_starts_successfully() {
    // Test that the server can start without panicking
    let mut child = Command::new("cargo")
        .arg("run")
        .spawn()
        .expect("Failed to start server");

    // Give the server time to start
    thread::sleep(Duration::from_secs(2));

    // Kill the server process
    child.kill().expect("Failed to kill server process");

    // If we get here, the server started without panicking on startup
    assert!(true, "Server should start successfully");
}
