## Pass if Less-Than-or-Equal (//flowstdlib/control/pass_if_lte.toml)
Let a piece of data pass, if it is less-than-or-equal to a control value

#### Include using
```
[[process]]
alias = "pilte"
source = "lib:://flowstdlib/control/pass_if_lte.toml"
```

#### Inputs
* `in` [Number] - numerical input to control the flow of
* `max` [Number] - numerical control value

[[output]]
name = "control"
type = "Number"

#### Outputs
* `passed` [Number] - `in` value if it is less than or equal to `max`
* `blocked` [Boolean] - `in` value if it more than (NOT less than or equal to) `max`
* `control` [Boolean] - `max` passed to the output each time