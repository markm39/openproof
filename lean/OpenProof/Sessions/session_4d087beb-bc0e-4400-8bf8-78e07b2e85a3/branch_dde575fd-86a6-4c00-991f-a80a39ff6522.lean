import Mathlib
/-!
OpenProof scratch session: Gcd Divides a Multiple of The Left Argument
Session id: session_4d087beb-bc0e-4400-8bf8-78e07b2e85a3
Mode: research
Updated: 
-/
-- branch: Gcd Divides a Multiple of The Left Argument repair [proving]
-- branch id: branch_dde575fd-86a6-4c00-991f-a80a39ff6522
-- node: Gcd Divides a Multiple of The Left Argument [verified]
-- kind: theorem
-- statement: theorem gcd_dvd_mul_right (m n k : Nat) : Nat.gcd m n ∣ m * k

-- artifact: Gcd Divides a Multiple of The Left Argument repair [pending]
-- artifact id: artifact_branch_dde575fd-86a6-4c00-991f-a80a39ff6522
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_dde575fd-86a6-4c00-991f-a80a39ff6522
-- label: Gcd Divides a Multiple of The Left Argument repair
-- metadata: {"hidden":true}

theorem gcd_dvd_mul_right (m n k : Nat) : Nat.gcd m n ∣ m * k := by
  rcases Nat.gcd_dvd_left m n with ⟨t, ht⟩
  refine ⟨t * k, ?_⟩
  calc
    m * k = (Nat.gcd m n * t) * k := by rw [ht]
    _ = Nat.gcd m n * (t * k) := by simp [Nat.mul_assoc]