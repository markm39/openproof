import Mathlib
/-!
OpenProof scratch session: Coprime Divisor Cancels From a Product Over ℕ
Session id: session_b9a220b4-8e3a-4c28-8be3-0801006ac43a
Mode: research
Updated: 
-/
-- branch: Coprime Divisor Cancels From a Product Over ℕ prover [proving]
-- branch id: branch_62e0f4ba-52cf-4f63-8c96-4db06cf1173c
-- node: Coprime Divisor Cancels From a Product Over ℕ [proving]
-- kind: theorem
-- statement: `theorem nat_coprime_dvd_of_dvd_mul {a b c : ℕ} (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by`

-- artifact: Coprime Divisor Cancels From a Product Over ℕ prover [pending]
-- artifact id: artifact_branch_62e0f4ba-52cf-4f63-8c96-4db06cf1173c
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_62e0f4ba-52cf-4f63-8c96-4db06cf1173c
-- label: Coprime Divisor Cancels From a Product Over ℕ prover
-- metadata: {"hidden":false,"foreground":true}

theorem nat_coprime_dvd_of_dvd_mul {a b c : ℕ}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  exact hcop.dvd_of_dvd_mul_right hdiv