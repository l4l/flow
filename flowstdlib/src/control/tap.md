## Tap (//flowstdlib/control/tap.toml)
Control the flow of data (flow or disapear it) based on a boolean control value.

#### Include using
```
[[process]]
alias = "tap"
source = "lib://flowstdlib/control/tap.toml"
```

#### Inputs
* `data` - the data flow we wish to control the flow if
* `control` - a boolean value to determine in `data` is passed on or not

#### Outputs
* `data` if `control` is true, nothing if `control` is false