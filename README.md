# Commity

Make writing commit messages **fun** again.

---

## Features

- **Customizable Fields**: Define the structure and content of your commit messages with a small configuration file.
- **Cross-Platform Support**: Use Commity seamlessly on Linux, macOS, and Windows.
- **Flexible Templates**: Generate commit messages tailored to your preferences using Go’s powerful templating engine.

---

## Configuration

Commity uses a simple yaml file to define how your commit messages are structured. This file can either be placed in the repository as hidden file `.commity.yaml` or in the user data directory:

- **Linux**: `$HOME/.config/commity/commity.yaml`
- **macOS**: `$HOME/Library/Application/commity/commity.yaml`
- **Windows**: `%AppData%\commity\commity.yaml`

### Sections

The `.commity.yaml` file consists of two main sections:

#### 1. `entries`

`entries` define the fields that Commity will prompt you to fill in. Each entry has the following properties:

- **`type`**: Specifies the type of the field. Can be one of the following:
  - `choice`: A **predefined** selection (similar to a dropdown or radio buttons)
  - `boolean`: A simple Yes/No field
  - `text`: A single-line or multi-line text input field
- **`name`**: The identifier for the field, used to reference its value in the commit message template
- **`label`**: The label displayed in the Commity UI
- **`description`**: Additional information displayed in the UI
- **`default`**: The default value for the field (optional)
- **`store`**: Wheter or not to cache this field for the next time

##### Extended Properties for Field Types

- **`choice`**
  - The user selects between predefined options
  - Offers a list `choices` which represent the individual options the user can select
    - `value`: The value used in the commit message
    - `label`: The label displayed in the UI
- **`boolean`**
  - A simple Yes/No field
- **`text`**
  - Allows users to input text
  - Additional properties:
    - `multiLine`: (Boolean) Enables multi-line text input
    - `minLength`: Minimum length of the input (0 = no restriction)
    - `maxLength`: Maximum length of the input (0 = no restriction)
    - `pattern`: Regular Expression to validate
    - `patternHint`: Validation hint shown if the regular expression does not match

#### 2. `template`

The `template` section is a string that defines how the commit message is generated.

Commity uses Go’s [text/template](https://pkg.go.dev/text/template) engine for rendering. You can reference any `entry` value by using the `name` defined in the configuration.

For example if you have a text field named `title` you can refer to it by using `{{ .title }}` in the template string. It is also possible to conditionally render something by using `{{ if .<field_name> }}<render this>{{ end }}`.

#### 3. `overview`

Boolean wheter or not to render an initial overview (Repository path and staged files).

### Example

```yaml
entries:
  - type: Choice
    name: type
    label: Commit Type
    description: What are you committing?
    default: feat
    choices:
      - value: feat
        label: This commit introduces a new feature
      - value: fix
        label: This commit fixes a bug
      - value: docs
        label: Everything related to documentation
      - value: test
        label: Everything related to testing
      - value: ci
        label: Modifications on the build system or steps of the CI pipeline
      - value: perf
        label: Performance improvements
      - value: refactor
        label: Refactoring a specific section of the codebase
      - value: revert
        label: Reverting existing code
      - value: style
        label: Code style or non functional modifications
      - value: chore
        label: Regular code maintenance. Should only be sparsely used if nothing else applied
    store: true

  - type: Text
    name: header
    label: Commit Header
    description: What did you change?
    minLength: 10
    maxLength: 50
    multiLine: false
    default: ""

  - type: Text
    name: body
    label: Commit Body
    description: What are the details of your changes?
    minLength: 0
    maxLength: 0
    multiLine: true
    default: ""

  - type: Boolean
    name: breaking_change
    label: Breaking Change
    description: Are you commiting breaking changes?
    default: false

template: |
  {{ .type }}{{ if .breaking_change }}!{{ end }}: {{ .header }}{{ if .body }}
  {{ .body }}{{ end }}

overview: true
```
