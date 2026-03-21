import Mathlib
/-!
OpenProof scratch session: Gcd Divides Left Factor Times k
Session id: session_ea38dbd4-26c3-41f9-8cd2-c053c45055e2
Mode: research
Updated: 
-/
-- branch: Gcd Divides Left Factor Times k planner [proving]
-- branch id: branch_8e84c5f5-b2cc-4f9d-8361-f1cfbd35d28f
-- node: Gcd Divides Left Factor Times k [proving]
-- kind: theorem
-- statement: theorem gcd_mul_right_dvd_left {m n k : Nat} : Nat.gcd m n ∣ m * k

-- artifact: Gcd Divides Left Factor Times k planner [pending]
-- artifact id: artifact_branch_8e84c5f5-b2cc-4f9d-8361-f1cfbd35d28f
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_8e84c5f5-b2cc-4f9d-8361-f1cfbd35d28f
-- label: Gcd Divides Left Factor Times k planner
-- metadata: {"hidden":true}

theorem gcd_mul_right_dvd_left {m n k : Nat} : Nat.gcd m n ∣ m * k := by
  rcases Nat.gcd_dvd_left m n with ⟨t, ht⟩
  refine ⟨t * k, ?_⟩
  calc
    m * k = (Nat.gcd m n * t) * k := by rw [ht]
    _ = Nat.gcd m n * (t * k) := by simp [Nat.mul_assoc]