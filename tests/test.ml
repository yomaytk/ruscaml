1 + 3 * 4 + 5;;
if 1 then 4 else 4;;
(x + 1) * 2 + (3 + 1);;
fun a -> a+5;;
let a = 1 in a;;
let a = 1 in let b = 3 in a+b*3;;
let rec f = fun a -> a+4 in 5;;
loop l = let a = 32 in a in 43+6;;
f a b c;;
let a = true in let b = false in f a b;;
(1+2*3);;
let a = (1 + 3 *4) in a;;
let a = (0, (1, (2, (3, 4)))) in a.2.2.2.1;;
let a = 1 in
let b = 43 in 
a+b;;
let a = 3 in loop b = 3 in recur b;;
let rec f = fun x -> x+1 in loop d = 3 in recur (f 5);;
if true then loop f = 4 in recur 4 else 5;;
(fun x -> fun y -> x + y) 2 3;;
fun x -> fun y -> x + y;;
let a = 1 in
let b = 2 in
let c = 3 in
let rec g = fun x -> a * x + c + d in
g 4;;
let rec h = fun x -> h (x+1) in
h 0;;
(fun x -> x+12) 23;;

(* recur check violataion *)
let a = 4 in recur 5;;

(* Compile Error Syntax *)
let 3 = 5 in 4;;
let a = 1 in
let b = 2 in
let c = 3 in 
let 3 = 4 in 
5;;