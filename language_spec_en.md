# Flap Language Specification

Flap is a minimal yet powerful stack-based esoteric programming language (Esolang).

## 1. Data Types

Flap handles a single data type: **64-bit signed integer (`i64`)**.
Strings are represented on the stack as a sequence of integers (ASCII codes).

- **Character**: ASCII characters (internally stored as integers)

## 2. Syntax Rules

### 2.1 Tokenization

- **Numbers**: Sequential digits are interpreted as a single integer.
- **Commands**: Any non-whitespace character (with a few exceptions) that is not a digit is interpreted as a single-letter command.
- **Separator (`,`)**: Used to explicitly separate numbers from other numbers or commands.
  - Example: `10,20+` (Push 10, Push 20, Add)
- **Whitespace**: Half-width spaces, tabs, and newlines are ignored and serve only as separators.

### 2.2 Comments

- Any content enclosed by `=` is treated as a comment and ignored during execution.
  - Example: `10 = this is a comment = 20 +`

---

## 3. Command List

### 3.1 Arithmetic Operations

Pops the top two values and pushes the result.

| Command | Name | Description                             | Stack Change (a, b -> Result) |
| :-----: | :--- | :-------------------------------------- | :---------------------------- |
|   `+`   | Add  | Addition                                | `a, b -> a + b`               |
|   `-`   | Sub  | Subtraction                             | `a, b -> a - b`               |
|   `*`   | Mul  | Multiplication                          | `a, b -> a * b`               |
|   `/`   | Div  | Division (Pushes 0 on division by zero) | `a, b -> a / b`               |
|   `%`   | Mod  | Modulo (Pushes 0 on division by zero)   | `a, b -> a % b`               |

### 3.2 Stack Operations

Directly manipulates the state of the stack.

| Command | Name | Description                 | Stack Change             |
| :-----: | :--- | :-------------------------- | :----------------------- |
|   `:`   | Dup  | Duplicate the stack top     | `a -> a, a`              |
|   `;`   | Pop  | Discard the stack top       | `a -> (none)`            |
|   `x`   | Swap | Swap the top two values     | `a, b -> b, a`           |
|   `@`   | Rot  | Rotate the top three values | `a, b, c -> b, c, a`     |
|   `R`   | Rev  | Reverse the entire stack    | `[1, 2, 3] -> [3, 2, 1]` |

### 3.3 Input/Output

| Command | Name          | Description                                                                                                                      | Stack Change        |
| :-----: | :------------ | :------------------------------------------------------------------------------------------------------------------------------- | :------------------ |
|   `r`   | Read Num      | Reads a whitespace-separated number from stdin and pushes it to the stack. Errors if input is not a number. **Pushes 0 on EOF.** | `-> n`              |
|   `t`   | Readable Char | Reads one local character from stdin and pushes its ASCII code to the stack. **Pushes 0 on EOF.**                                | `-> c`              |
|   `T`   | Text          | Read one line from standard input and push characters as codes. **The last character ends up at the top.**                       | `-> c1, c2, ... cn` |
|   `p`   | Print         | Pop an integer and print it to standard output                                                                                   | `n -> (none)`       |
|   `P`   | PChar         | Pop a character code and print it as a character                                                                                 | `n -> (none)`       |

---

## 4. Control Flow

Controls execution flow based on the state of the stack.

### 4.1 Iteration (`[` ... `]`)

- **While Loop**
- When `[` is reached: If the stack top is **non-zero**, execute the block.
- When `]` is reached: If the stack top is **non-zero**, jump back to just after the corresponding `[`.
- **Note**: The top value is **not popped** at the start or end of the loop.

### 4.2 Conditional Branching (`(` ... `)`)

- **If Statement**
- When `(` is reached: If the stack top is **non-zero**, execute the block.
- When `(` is reached: If the stack top is **zero**, jump to the corresponding `)`.
- **Important**: The condition value is **always popped** once completion/jump happens, regardless of whether the block was executed.

---

## 6. Limitations

- The current version only supports **ASCII characters (0-255)**.
- Multi-byte characters (Japanese, etc.) are not correctly handled during input or output.
