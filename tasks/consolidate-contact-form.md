# Implementation Plan: Consolidating Contact Form Templates

## Overview

Currently, the contact form uses 3 separate templates. We want to consolidate everything into `contact_form.html` using Datastar signals to show/hide success and error messages.

---

## Key Datastar Concepts You'll Use

| Concept | What It Does | Example |
|---------|--------------|---------|
| `data-signals` | Define reactive state variables | `data-signals="{showSuccess: false}"` |
| `data-show` | Show/hide element based on signal | `data-show="$showSuccess"` |
| `data-text` | Set text content from signal | `data-text="$errorMessage"` |
| `data-bind` | Two-way binding for form fields | `data-bind="name"` |
| `PatchSignals` | Backend sends signal updates via SSE | `PatchSignals::new(json_string)` |

---

## Step 1: Template Changes (`templates/contact_form.html`)

### 1.1 Add signals definition to the wrapper div
Define signals for visibility control, error message, and form fields:
- `showSuccess`, `showError` - booleans for visibility
- `errorMessage` - string for error text
- `name`, `email`, `phone`, `service`, `message` - form field values

### 1.2 Add success message section (hidden by default)
- Copy content from `contact_success.html`
- Add `data-show="$showSuccess"` to control visibility
- Change "Send Another Message" button to reset signals: `data-on-click="$showSuccess = false; $showError = false"`

### 1.3 Add error message section (hidden by default)
- Add `data-show="$showError"` to control visibility
- Use `data-text="$errorMessage"` to display error text

### 1.4 Wrap the form with visibility control
- Add `data-show="!$showSuccess"` so form hides when success shows

### 1.5 Add two-way binding to all form inputs
- Add `data-bind="name"`, `data-bind="email"`, etc. to each field

---

## Step 2: Backend Changes (`src/main.rs`)

### 2.1 Add import
```rust
use datastar::prelude::{PatchElements, PatchSignals};
```

### 2.2 Modify `contact_submit_handler`
Instead of rendering templates, send signal updates:

**On success** - send these signals:
- `showSuccess: true`, `showError: false`
- Reset form fields to empty strings

**On error** - send these signals:
- `showError: true`, `showSuccess: false`
- `errorMessage: "the validation error"`
- Don't reset form fields (preserve user input)

### 2.3 Simplify `ContactFormError` struct
Only needs `error_message` field now (form values managed by frontend)

### 2.4 Remove unused code
- Delete `ContactSuccessTemplate` struct
- Delete `ContactFormErrorTemplate` struct
- Delete `create_success_template` function
- Delete `ContactFormError::to_sse_response` method

---

## Step 3: Delete Obsolete Templates

| File to Delete | Reason |
|----------------|--------|
| `templates/contact_success.html` | Content moved into contact_form.html |
| `templates/contact_form_error.html` | Content moved into contact_form.html |

---

## Step 4: Update Tests (`tests/contact_integration_test.rs`)

- Remove CSS-based alert visibility tests (no longer applicable)
- Add tests to verify Datastar signal structure in template:
  - Check `data-signals` is defined
  - Check `data-show="$showSuccess"` exists
  - Check `data-show="$showError"` exists
  - Check `data-text="$errorMessage"` exists
  - Check form fields have `data-bind`

---

## Step 5: Verification Checklist

1. **Build**: `cargo build` - no errors
2. **Tests**: `cargo test` - all pass
3. **Manual test** - initial state: form visible, messages hidden
4. **Manual test** - validation error: error shows, form fields preserved
5. **Manual test** - success: success shows, form hidden, fields cleared
6. **Manual test** - "Send Another Message": form reappears empty

---

## Common Pitfalls

1. **Escape quotes in error messages** - use `.replace('"', "\\\"")`
2. **Always set opposite signal** - when `showSuccess: true`, also set `showError: false`
3. **Reset form fields on success** - send empty strings for all field signals
4. **Use `$` prefix in templates** - `data-show="$showSuccess"` not `data-show="showSuccess"`

---

## Files Summary

| File | Action |
|------|--------|
| `templates/contact_form.html` | Modify |
| `templates/contact_success.html` | Delete |
| `templates/contact_form_error.html` | Delete |
| `src/main.rs` | Modify |
| `tests/contact_integration_test.rs` | Modify |
