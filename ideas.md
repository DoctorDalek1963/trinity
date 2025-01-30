# DSL upgrades

We should support vectors in the DSL and parser. (Should we also allow numeric variables? They'd both have to use lowercase.) I think matrices and vectors should be able to work together and be evaluated type-dynamically and we just reject the result at the end if it's the wrong type (vector when expecting matrix, etc.)

Upgrade dimension. The `!` character should upgrade from 2D to 3D, so `[a b; c d]! == [a b 0; c d 0; 0 0 1]` and `[x; y]! == [x; y; 0]`.

Downgrade dimension. The `?` character should downgrade from 3D to 2D, but only if the z component is correct. So `[a b 0; c d 0; 0 0 1]? == [a b; c d]` and `[x; y; 0]? == [x; y]` but we should reject anything else. So the `?` doesn't project a vector into 2D, it just takes a vector that's already in the xy plane and ignores the 0 z component. Likewise `?` requires a 3D matrix that leaves z untouched.

## To consider

What about non-square matrices like `[1 0 0; 0 1 0]`? This one projects a 3D vector into the xy plane. Should this be supported or should we require `([1 0 0; 0 1 0; 0 0 0] v)?` instead? In the second case, it might be useful to allow custom function definitions, like `func ProjectXY(v) {([1 0 0; 0 1 0; 0 0 0] v)?}`.

If we allow functions, should it be a completely separate system, like a dialog box that prompts for function name, variable bindings, and function body separately? I think so, or else we'd need a new syntax for function definition. And then where is it done? Inline with other expressions? Either way we'd need a custom character to prefix function calls to avoid ambiguity with named vectors and matrices, like `@ProjectXY(v)`. But then should `rot` be treated specially anymore? Perhaps real functions for rotation would be good, but they'd have to be defined natively, which is fine.

And then what about scope bindings? Should functions be able to access variables defined globally?

# Restart work on Bevy

Scrap and start again from the ground up. Maybe make other, real games first to learn Bevy properly. We should have one consistent game state that supports 2D and 3D interchangeably and the user can just change the camera angle. This system works well with upgrading and downgrading dimensions.
