import Mathlib
/-!
OpenProof scratch session: Gcd Divides The Left Factor After Right Multiplication
Session id: session_0cc1cbeb-167b-4d9f-b4d1-8549ea53bf45
Mode: research
Updated: 
-/
-- branch: Gcd Divides The Left Factor After Right Multiplication planner [proving]
-- branch id: branch_102bf145-070c-4fff-aa67-7de4239b3d46
-- node: Gcd Divides The Left Factor After Right Multiplication [proving]
-- kind: theorem
-- statement: theorem gcd_mul_right_dvd_left {m n k : Nat} : Nat.gcd m n ∣ m * k

-- artifact: Gcd Divides The Left Factor After Right Multiplication planner [pending]
-- artifact id: artifact_branch_102bf145-070c-4fff-aa67-7de4239b3d46
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_102bf145-070c-4fff-aa67-7de4239b3d46
-- label: Gcd Divides The Left Factor After Right Multiplication planner
-- metadata: {"hidden":true}

theorem gcd_mul_right_dvd_left {m n k : Nat} : Nat.gcd m n ∣ m * k := by
  exact dvd_mul_of_dvd_left (Nat.gcd_dvd_left m n) k