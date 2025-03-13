# cbsub

[![Rust](https://github.com/db-scribe/cbsub/actions/workflows/rust.yml/badge.svg)](https://github.com/db-scribe/cbsub/actions/workflows/rust.yml)

**cbsub** is a command-line utility written in Rust that processes a text file containing variables and substitutes them with specified values before copying the final output to the clipboard. It supports both multiple and single variable substitutions, previewing results, and listing variables in the input file.

## Features

- **Variable Substitution:** Replace placeholders (e.g. `{{code}}`, `{{name}}`) in a text file with provided values.
- **Multiple Substitutions:** Use the `-s` flag to supply multiple key-value pairs.
- **Single Substitution:** Provide a positional argument if the file contains exactly one variable.
- **Preview Mode:** Use the `-p` flag to preview the processed text without copying it to the clipboard.
- **List Variables:** Use the `-l` flag to list all variables detected in the file.
- **Clipboard Integration:** Automatically copies the processed content to the system clipboard:
  - **macOS:** Uses `pbcopy`
  - **Windows:** Uses `clip`
  - **Linux:** Uses `xclip` (make sure it's installed)

## Installation

### 1. Clone the Repository

```sh
git clone <repository-url>
cd cbsub
```

### 2. Install Clipboard Dependencies

- **macOS:** `pbcopy` is available by default.
- **Windows:** `clip` is available by default.
- **Linux:** Install `xclip` (or adjust the code to use `xsel` if preferred):

  ```sh
  sudo apt-get install xclip
  ```

### 3. Build the Project

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed, then build the project using Cargo:

```sh
cargo build --release
```

The compiled binary will be located in the `target/release` directory.

## Usage

```sh
cbsub <file> [options]
```

### Copy File to Clipboard (No Variables)

If the file contains no variables, `cbsub` copies its contents directly to the clipboard.

```sh
cbsub my_prompt.txt
```

### Substitute Multiple Variables

Use the `-s` flag to substitute variables with the desired values.

```sh
cbsub my_prompt.txt -s code="1234" -s name="John Doe"
```

**Expected Output:**

- The processed file with variables replaced is copied to the clipboard.

### Substitute a Single Variable (Positional Argument)

If the file contains exactly one variable, you can provide the substitution value as a positional argument.

```sh
cbsub my_prompt.txt "1234"
```

If the file contains more than one variable, an error is displayed.

### Preview Substituted Output

Use the `-p` flag to preview the processed text without copying it to the clipboard.

```sh
cbsub my_prompt.txt -s code="1234" -p
```

### List Variables in a File

To list all variables detected in the file, use the `-l` flag.

```sh
cbsub my_prompt.txt -l
```

**Example Output:**

```text
Found variables:
 - {code}
 - {name}
```

## Example

### Sample Prompt File (`my_prompt.txt`)

```text
Hello {{name}},

Your verification code is {{code}}. Please enter this code to proceed.

Best,
Support Team
```

### Example Command

```sh
cbsub my_prompt.txt -s code="9876" -s name="Alice"
```

**Final Output (Copied to Clipboard):**

```text
Hello Alice,

Your verification code is 9876. Please enter this code to proceed.

Best,
Support Team
```

## Testing

A comprehensive set of tests is provided. To run the tests, execute:

```sh
cargo test
```

This will run unit tests for variable extraction, substitution processing, parsing, and error conditions.

## Contributing

Contributions are welcome! If you have ideas or improvements, please open an issue or submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
