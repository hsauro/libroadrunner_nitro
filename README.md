# libroadrunner_nitro

A fast engine for simulating biochemical models (SBML), written in Rust. It does the same job as
[libroadrunner](https://github.com/sys-bio/roadrunner), gives the same answers, and needs neither libroadrunner
nor libsbml.

This was developed with the assistance of Claude Code (Opus 5.5, med)

You can use it two ways:

| If you write... | Use | Section |
|---|---|---|
| Python | the Python package `roadrunner_nitro` | [Use it from Python](#use-it-from-python) |
| C, C++, MATLAB, Julia, C#, R, or anything that can load a C library | the C library `libroadrunner_nitro_c_api`, which has all 350 functions of libroadrunner's C API | [Use it from C and other languages](#use-it-from-c-and-other-languages-the-c-library) |

It is the engine inside [Tellurium Nitro](https://github.com/gshinkim/tellurium-nitro). If you want the full
tellurium experience in Python (`te.loada(...)`, plotting, SED-ML, COMBINE archives), install Tellurium Nitro
instead; it includes this engine.

Your existing `libroadrunner` is not touched: this package has its own name (`roadrunner_nitro`), so both can be
installed side by side.

Tested on macOS (Apple Silicon). Linux and Windows should work but are untested.

---

## Step 1: Install the tools (once)

Open a terminal (macOS: the Terminal app; Windows: PowerShell) and check each of these:

1. **git**: `git --version`. If missing: on macOS run `xcode-select --install`; on Windows get it from
   https://git-scm.com.
2. **Python 3.9 or newer** (only needed for the Python package): `python3 --version`. If missing:
   https://python.org.
3. **Rust**, which compiles the engine: `cargo --version`. If missing:
   - macOS / Linux: run `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`, then close and
     reopen the terminal.
   - Windows: download and run `rustup-init.exe` from https://rustup.rs. When it asks to install the
     "Visual Studio C++ Build Tools", say yes.

## Step 2: Download it

```bash
git clone https://github.com/gshinkim/libroadrunner_nitro.git
cd libroadrunner_nitro
```

Run all the commands below from inside this `libroadrunner_nitro` folder.

---

## Use it from Python

### Install

Make a fresh virtual environment (a private folder of Python packages, so nothing else on your computer changes)
and install into it. The build takes 1-3 minutes the first time.

```bash
python3 -m venv .venv
source .venv/bin/activate          # Windows: .venv\Scripts\activate
pip install --upgrade pip
pip install .
```

Check it:

```bash
python -c "import roadrunner_nitro; print('ok')"
```

Each time you open a new terminal, run `source .venv/bin/activate` again before using it.

You can also skip the download and install straight from GitHub (Rust is still needed):
`pip install git+https://github.com/gshinkim/libroadrunner_nitro.git`

### Your first simulation

Save this as `first.py` and run `python first.py`:

```python
import roadrunner_nitro as rn

# Write the model in Antimony (a short text format) and turn it into SBML.
# To use your own SBML file instead: sbml = open('my_model.xml').read()
sbml = rn.antimony_to_sbml('''
    J1: $X0 -> S1; k1*X0     # S1 is made from X0 at rate k1*X0 ($ means X0 is held fixed)
    J2: S1 -> ;    k2*S1     # S1 is broken down at rate k2*S1
    X0 = 10; k1 = 0.5; k2 = 0.2
''')

r = rn.RoadRunner(sbml)

result = r.simulate(0, 10, 6)        # from time 0 to 10, 6 time points
print(r.timeCourseSelections)        # what the columns are: ['time', '[S1]']
print(result)                        # a numpy array, one row per time point

r.steadyState()                      # find where the system settles
print('S1 at steady state:', r['S1'])                  # 25.0
print('control of k1 over J2:', r.getCC('J2', 'k1'))   # 1.0

r['k1'] = 1.0                        # change a parameter
r.reset()                            # put S1 back to its starting amount (k1 stays 1.0)
print(r.simulate(0, 10, 6))
```

The method names are libroadrunner's: `simulate`, `steadyState`, `getCC`, `getValue`, `setValue`,
`getFloatingSpeciesIds`, `getReactionRates`, `reset`, and so on. One difference: functions that return a matrix
(for example the Jacobian or the stoichiometry matrix) give back three things, `(array, row names, column names)`.

---

## Use it from C and other languages (the C library)

Many programs use libroadrunner through its C library instead of Python: C and C++ programs, MATLAB, Julia, R, C#,
Delphi and others. This repository builds a drop-in replacement, `libroadrunner_nitro_c_api`. It has **all 350
functions** of libroadrunner's C API (`rrc_api.h`, `rrc_types.h`, `rrc_utilities.h`) with the same names,
arguments and memory rules. It does not need Python, libroadrunner or libsbml.

### Build the library

Needs only git and Rust (Step 1). Takes 1-3 minutes.

```bash
cargo build --release -p roadrunner_nitro_c_api
```

This makes one file in the folder `target/release/`:

| System | File |
|---|---|
| macOS | `libroadrunner_nitro_c_api.dylib` |
| Linux | `libroadrunner_nitro_c_api.so` (untested) |
| Windows | `roadrunner_nitro_c_api.dll` (untested) |

The header files you `#include` are in `capi/include/`. They are libroadrunner's own headers, unchanged.

The library is tuned for the processor of the computer that built it. To use it on a different computer, delete
the `.cargo` folder first and build again; that makes a library that runs on any computer of the same type.

**Check it** (optional, needs Python): this confirms the library has every function in the headers.

```bash
python3 capi/tests/check_exports.py       # prints: 350 declared / 350 exported
```

### Use it from C or C++

A complete example is in [`capi/examples/simulate.c`](capi/examples/simulate.c). It loads a model, simulates it,
and finds the steady state and a control coefficient. The core of a program looks like this:

```c
#include <stdio.h>
#include "rrc_api.h"

int main(void) {
    RRHandle rr = createRRInstance();
    if (!loadSBMLFromFile(rr, "model.xml")) {          /* or loadSBML(rr, sbml_text) */
        printf("error: %s\n", getLastError());
        return 1;
    }
    RRCDataPtr result = simulateEx(rr, 0, 10, 100);    /* time 0 to 10, 100 points */
    double value;
    getRRCDataElement(result, 99, 1, &value);          /* last row, column 1 (column 0 is time) */
    printf("%g\n", value);
    freeRRCData(result);                               /* free what the library gives you, as with libroadrunner */
    freeRRInstance(rr);
    return 0;
}
```

Compile and run the example (macOS and Linux):

```bash
cc -I capi/include capi/examples/simulate.c \
   -L target/release -lroadrunner_nitro_c_api -Wl,-rpath,$PWD/target/release -o simulate
./simulate                  # uses the model built into the example
./simulate my_model.xml     # or any SBML file
```

What the flags mean: `-I capi/include` finds the headers; `-L target/release -lroadrunner_nitro_c_api` links the
library; `-Wl,-rpath,...` tells the program where to find the library when it runs. With the built-in model it
prints a time course and then `steady state S1 = 25 (expected 25), control coefficient of J2 on k1 = 1 (expected 1)`.

On Windows (untested), from the "Developer Command Prompt for Visual Studio": compile with
`cl /I capi\include capi\examples\simulate.c target\release\roadrunner_nitro_c_api.dll.lib`, and keep
`roadrunner_nitro_c_api.dll` in the same folder as `simulate.exe`.

### Use it from Python through the C library

Python's built-in `ctypes` can call the C library directly, with no Python package installed:

```python
import ctypes
rr = ctypes.CDLL("target/release/libroadrunner_nitro_c_api.dylib")   # .so on Linux, .dll on Windows
rr.createRRInstance.restype = ctypes.c_void_p
rr.loadSBMLFromFile.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
rr.simulateEx.restype = ctypes.c_void_p
rr.simulateEx.argtypes = [ctypes.c_void_p, ctypes.c_double, ctypes.c_double, ctypes.c_int]
rr.getRRDataNumRows.argtypes = [ctypes.c_void_p]
rr.getRRCDataElement.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_int, ctypes.POINTER(ctypes.c_double)]
rr.freeRRCData.argtypes = [ctypes.c_void_p]
rr.freeRRInstance.argtypes = [ctypes.c_void_p]

h = rr.createRRInstance()
rr.loadSBMLFromFile(h, b"model.xml")
res = rr.simulateEx(h, 0.0, 10.0, 11)
v = ctypes.c_double()
rr.getRRCDataElement(res, rr.getRRDataNumRows(res) - 1, 1, ctypes.byref(v))
print(v.value)
rr.freeRRCData(res)
rr.freeRRInstance(h)
```

### Use it from MATLAB, Julia, C# and others

Load the library file the same way you would load libroadrunner's, with the headers from `capi/include`
(these were not tested here):

- **MATLAB**: `loadlibrary('libroadrunner_nitro_c_api', 'rrc_api.h', 'addheader', 'rrc_types.h')`, then
  `h = calllib('libroadrunner_nitro_c_api', 'createRRInstance')`.
- **Julia**: `h = ccall((:createRRInstance, "libroadrunner_nitro_c_api"), Ptr{Cvoid}, ())`.
- **C#**: `[DllImport("roadrunner_nitro_c_api")] static extern IntPtr createRRInstance();`

### Switch an existing libroadrunner program to this engine

Either rebuild the program with `-lroadrunner_nitro_c_api` instead of `-lroadrunner_c_api`, or, without
rebuilding, give this library libroadrunner's file name and put it where the program looks for libroadrunner.
On macOS:

```bash
cp target/release/libroadrunner_nitro_c_api.dylib /path/to/libroadrunner_c_api.dylib
install_name_tool -id @rpath/libroadrunner_c_api.dylib /path/to/libroadrunner_c_api.dylib
```

The program then runs on this engine unchanged.

### Where it differs from libroadrunner's C API

Only where libroadrunner's behaves wrongly:

| Function | libroadrunner | This library |
|---|---|---|
| `getRatesOfChange` | values in a different order from `getRatesOfChangeIds` | values match the ids |
| `setTimeCourseSelectionList("time, cc(S1, k1)")` | splits inside `cc(...)` at the comma | splits only between selections |
| `loadSimulationSettings` | settings have no effect | start, duration, steps and tolerances are applied |
| `getFloatingSpeciesInitialConcentrationByIndex` | a bad index returns "success" without a value | a bad index is an error |
| `getConfigurationXML`, `writeRRData` | declared but missing | implemented |
| `steadyState` | gives up when Newton's method fails from the current state | also finds steady states Newton alone misses |
| euler / rk4 / rk45 integrators | ignore events after the start time | apply every event |

---

## Troubleshooting

- **`cargo: command not found`**: Rust isn't installed, or the terminal was opened before installing it. Do
  Step 1 again, then open a new terminal.
- **`pip install .` fails**: read the last lines of the error. Usually Rust is missing (above) or Python is
  older than 3.9.
- **`No module named roadrunner_nitro`**: the virtual environment isn't active. Run
  `source .venv/bin/activate` (Windows: `.venv\Scripts\activate`).
- **A C program says `Library not loaded` or `cannot open shared object file`**: it can't find the library
  file. Compile with the `-Wl,-rpath,...` flag shown above, or copy the library file next to the program.

## Tests

- **C library**: build it (above), then `pip install pytest` and run `python -m pytest capi/tests`. These run
  on their own (21 tests, including a C program checked for memory leaks).
- **Python package**: the tests in `tests/` compare results with stored libroadrunner results
  (`tests/reference/*.npz`). They also need `antimony` and files from
  [Tellurium Nitro](https://github.com/gshinkim/tellurium-nitro), so run them from Tellurium Nitro's `rrs/`
  folder, which holds this same code. `tests/test_sbml_conversion.py` reads the SBML Test Suite from the folder
  named in the `SBML_SEMANTIC_DIR` environment variable.

## Credits

`crates/antimony` and `crates/sbml-io` come from
[sys-bio/PathwayDesigner-rs](https://github.com/sys-bio/PathwayDesigner-rs) (see `crates/UPSTREAM`). The C
headers in `capi/include/` come from [sys-bio/roadrunner](https://github.com/sys-bio/roadrunner) (Apache 2.0,
University of Washington). This repository is the `rrs/` folder of Tellurium Nitro, split out with its history.

## License

Apache License 2.0, see [`LICENSE`](LICENSE).
