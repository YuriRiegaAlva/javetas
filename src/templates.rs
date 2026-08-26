pub const REPO_URL: &str = "https://github.com/YuriRiegaAlva/javetas";

pub fn main_java(package: Option<&str>) -> String {
    let package_line = match package {
        Some(p) => format!("package {p};\n\n"),
        None => {
            String::from("// This project has no package: classes live directly inside src/.\n\n")
        }
    };
    format!(
        r#"// Comments start with // and are ignored by the compiler.
// They exist for you and for anyone else reading the code.

{package_line}/**
 * Main.java
 *
 * This is the entry point of the program: it contains the `main` method,
 * which is the first thing Java runs.
 *
 *   public  : anyone can call it
 *   static  : it belongs to the class, no object needed
 *   void    : it returns nothing
 *   String[] args : command-line arguments
 */
public class Main {{
    public static void main(String[] args) {{
        System.out.println("Hello, World!");

        // Try it: add your own lines below, then run:
        //   make run          (or `javetas run`)
    }}
}}
"#
    )
}

pub fn class_java(package: Option<&str>, name: &str) -> String {
    let package_line = match package {
        Some(p) => format!("package {p};\n\n"),
        None => {
            String::from("// This project has no package: classes live directly inside src/.\n\n")
        }
    };
    format!(
        r#"{package_line}/**
 * {name}.java
 *
 * A class is a blueprint for objects.
 *
 * - Fields store state (what the object knows).
 * - Methods define behavior (what the object can do).
 *
 * Add yours below, then use {name} from Main:
 *   {name} x = new {name}();
 */
public class {name} {{
}}
"#
    )
}

pub fn makefile(main_class: &str) -> String {
    format!(
        r#"# Shortcuts for compiling and running this project.
# Run `make`, `make run` or `make clean`.

JAVAC = javac
JAVA  = java
OUT   = out
SRC   = $(shell find src -name '*.java')

.PHONY: all build run clean

all: build

build: $(SRC)
	mkdir -p $(OUT)
	$(JAVAC) -d $(OUT) $(SRC)

run: build
	$(JAVA) -cp $(OUT) {main_class}

clean:
	rm -rf $(OUT)
"#
    )
}

pub fn config(package: Option<&str>) -> String {
    let package = package.map_or_else(String::new, |p| p.to_string());
    format!(
        r#"# javetas project configuration.
# package : Java package of the sources (empty = none)
# main    : class run by `javetas run` and `make run`
package={package}
main=Main
"#
    )
}

pub fn gitignore() -> String {
    r#"# Compiled classes
out/
*.class

# IDE files
.idea/
.vscode/
*.iml
"#
    .to_string()
}

pub fn readme(name: &str, main_class: &str) -> String {
    format!(
        r#"# {name}

A Java learning project created with [javetas]({REPO_URL}).

## Layout

```
{name}/
├── Makefile      # build / run / clean shortcuts
├── .javetas      # javetas config (package, main class)
├── .gitignore    # files git should ignore
└── src/          # your .java source files
```

## Requirements

- JDK 25 or newer. Check with `java -version` and `javac -version`.

## Compile and run

With the Makefile (Linux/macOS):

```sh
make          # compile
make run      # compile and run Main
make clean    # delete compiled classes
```

Or with javetas, from anywhere inside this project:

```sh
javetas build
javetas run
javetas run SomeOtherClass   # run a different class
```

## How it works (the learning part)

1. `make build` runs `javac -d out <all .java files under src/>`.
   `javac` compiles your source into `.class` bytecode inside `out/`.
2. `make run` runs `java -cp out {main_class}`.
   `java` starts the JVM and runs the `main` method of that class.
3. `-cp out` is the *classpath*: it tells Java where to look for
   compiled classes.

Try adding a second class and using it from Main:

```sh
javetas add Persona
```

Then edit `src/**/Main.java` to create a `Persona` and call its methods.
"#
    )
}
