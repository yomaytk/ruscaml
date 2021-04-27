EXPECT(1+3;; => 4):
EXPECT(1 + 3 * 4 + 5;; => 18):
EXPECT(if 1 then 4 else 6;; => 4):
EXPECT(let x = 3 in (x + 1) * 2 + (3 + 1);; => 12):
EXPECT(let a = 1 in let b = 3 in a+b*3;; => 10):
EXPECT(let rec f = fun a -> a+4 in 5;; => 5):
EXPECT(loop l = let a = 32 in a in 43+6;; => 49):
EXPECT(let f = fun x -> fun y -> fun z -> x+y+z in f 1 2 3;; => 6):
EXPECT(let f = fun x -> fun y -> x+y in let a = 3 in let b = 4 in f a b;; => 7):
EXPECT(let a = (0, (1, (2, (3, 4)))) in a.2.2.2.1;; => 3):
EXPECT(let a = 1 in
let b = 43 in 
a+b;; => 3):
EXPECT(loop v = (1, 0) in
if v.1 < 11 then
  recur (v.1 + 1, v.1 + v.2)
else
  v.2;; => 55):
EXPECT((fun x -> fun y -> x + y) 2 3;; => 3):
EXPECT(let a = 1 in
let b = 2 in
let c = 3 in
let d = 4 in
let rec g = fun x -> a * x + c + d in
g 4;; => 12):
EXPECT((fun x -> fun y -> x + y) 2 3;; => 3):
EXPECT(let a = let a = 1 in a+1 in a;; => 3):

(* recur check violataion *)
(* let a = 4 in recur 5;; *)

(* Compile Error Syntax *)
(* let 3 = 5 in 4;;
let a = 1 in
let b = 2 in
let c = 3 in 
let 3 = 4 in 
5;; *)