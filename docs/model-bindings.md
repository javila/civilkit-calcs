# Model bindings

A model binding ties one worksheet input to a quantity of a structural model, so a design sheet
can take its actions and properties from an analysis instead of having them typed in. This page
is the specification. calcpad-rs implements only part of it so far:

- **Implemented:** the parser (`calcpad-core/src/model_bindings.rs`) and the in-app display. A
  bound input carries a **model** tag with its binding path, and a malformed binding shows its
  error in place.
- **Not implemented:** connecting a model (Studio wiring), fetching values, provenance, the stale
  state and refusal. These are specified below so the sheets written now stay valid when they
  arrive.

With no model connected, a bound input is an ordinary input: the value typed in it is used, and
the tag says so.

## Syntax

A binding is a comment line placed **directly before** the input it binds (blank lines may sit
between them, nothing else):

```
'<!--civilkit: bind N_d = member.N_c @ ULS envelope -->
'Design axial compression N* -'N_d = ? {150}kN
```

```
binding   = "'<!--civilkit: bind " variable " = " path [ " @ " selector ] " -->"
path      = object "." quantity
selector  = "ULS envelope" | "SLS envelope" | "case " name | "combination " name
```

- **Comment form.** It is an HTML comment inside a Calcpad text line. Calcpad without bindings
  passes it to the output unseen, and the sheet still runs on its own.
- **Input only.** The bound line must be an input (`variable = ? {default}unit`). Binding a
  calculated variable is an error.
- **One binding per variable.** A variable bound twice is an error.

## Objects and quantities

The object is the model object the worksheet is attached to: the member being checked, its
nodes, the footing, the connection, and that member's section and material. The id of the
attached object is part of provenance (below), not of the path.

| Object | Quantity | Measures | Kind |
|---|---|---|---|
| member | `N_c`, `N_t` | greatest axial compression, tension (magnitudes) | action effect |
| member | `V_x`, `V_y` | shear force | action effect |
| member | `M_x`, `M_y` | greatest bending moment (magnitude) about the section axis | action effect |
| member | `T` | torsion | action effect |
| member | `M_x_end1`, `M_x_end2` | signed end moments (for α<sub>m</sub>, β<sub>m</sub>) | action effect |
| member | `w` | uniformly distributed load intensity | action effect |
| member | `delta` | greatest deflection relative to the chord | action effect |
| member | `length`, `L_y`, `L_b` | length, spacing of minor-axis restraints, segment length for lateral-torsional buckling | property |
| member | `k_e_x`, `k_e_y` | effective length factors | property (ratio) |
| node | `R_x`, `R_y`, `R_z`, `M_rx`, `M_ry`, `M_rz` | reactions | action effect |
| node | `D_x`, `D_y`, `D_z` | displacements | action effect |
| footing | `N`, `V`, `M_x`, `M_y` | column actions at the footing | action effect |
| footing | `B_x`, `B_y`, `D`, `column_b_x`, `column_b_y` | plan size, thickness, column size | property |
| connection | `N`, `V`, `M` | actions at the connection | action effect |
| section | `d`, `b_f`, `t_f`, `t_w`, `b`, `D`, `r_x`, `r_y` | lengths | property |
| section | `A_g` | gross area | property |
| section | `I_x`, `I_y`, `J` | second moments, torsion constant | property |
| section | `Z_x`, `Z_y`, `S_x`, `S_y` | elastic and plastic moduli | property |
| section | `I_w` | warping constant | property |
| material | `f_y`, `f_y_flange`, `f_y_web`, `f_u`, `E`, `G`, `f_c` | stresses and moduli | property |

- **Unknown names are errors.** An unknown object or quantity is an error, never a silent default.
- **Extending the set.** The table is the list in `QUANTITIES` in the parser. To add a quantity,
  add it there and here.

## Selectors

The selector says which results an action effect comes from. It is required for an action effect
and not allowed for a property.

| Selector | Takes |
|---|---|
| `ULS envelope` | the extreme over every strength combination |
| `SLS envelope` | the extreme over every serviceability combination |
| `case <name>` | one load case, e.g. `case G`, `case Wu` |
| `combination <name>` | one combination, e.g. `combination 1.2G+1.5Q` |

**Envelopes are not concurrent.** An envelope takes each quantity's extreme on its own, so the
`N_c` and `M_x` of one sheet may come from different combinations. That is conservative for an
interaction check only when both extremes govern together. A check that needs concurrent actions
binds each input to the same `combination`, and runs once per combination.

## Units

A bound value is always a number with a unit, converted to the units of the input it replaces
(`member.N_c` in kN into an input written in kN, or in N; a mismatch of dimension is an error).

The parser checks the input now:

- **Dimensioned quantity:** the input must carry a unit.
- **Ratio:** the input must carry none.

## Provenance (specified, not yet implemented)

Each bound input shows, next to its value:

- **Object:** the model object id, e.g. `member 12`, `node 4`.
- **Results:** the combination or load case the value came from. For an envelope, this is the
  governing one.
- **Analysis run:** its id and time.

Provenance is printed with the sheet, so a printed calculation says where every bound number came
from.

## Stale state (specified, not yet implemented)

When the model changes after a sheet has taken its values (an edit, a new analysis, a deleted
combination), every bound input is marked **stale**, and so is every result that depends on it.
The sheet keeps the old numbers visible but marked, until it is refreshed from the new run.

## Refusal (specified, not yet implemented)

When a model is connected but cannot supply a bound value, the sheet refuses; it does not fall
back to the typed default. Typical causes:

- no analysis results;
- the object is missing;
- the selector names a case or combination that does not exist;
- the quantity is not defined for that object type.

A refusal works like this:

- The input is marked **missing**, with the reason.
- Every check that depends on it is refused and shows no ratio.
- The sheet's summary names the missing inputs.

**The default is never used silently while a model is connected.** It is the value for a sheet
run on its own, not a fallback.

## Display (implemented)

`model_bindings::annotate_html` runs after the render in calcpad-wasm (the browser app). The CLI
output is unchanged.

- **Bound input.** After each binding comment in the output, a tag goes at the end of the next
  paragraph, which is the input line: **model** `member.N_c @ ULS envelope`. It carries the
  attributes `data-bind-var`, `data-bind-path` and `data-bind-selector` for later wiring, and a
  title noting that no model is connected.
- **Malformed binding.** The tag reads **model binding error** with the reason.
- **Unrendered branch.** A binding in a branch that was not rendered is not shown.

