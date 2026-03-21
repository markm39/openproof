import Mathlib
/-!
OpenProof scratch session: Nat.Coprime Divides Cancellation on ℕ
Session id: session_527e5949-bc7c-481f-83ee-8aa556b1a49d
Mode: research
Updated: 
-/
-- branch: Nat.Coprime Divides Cancellation on ℕ repair [proving]
-- branch id: branch_4764c2dd-6914-4313-9b0d-7f328ea61e91
-- node: Nat.Coprime Divides Cancellation on ℕ [proving]
-- kind: theorem
-- statement: theorem coprime_dvd_of_dvd_mul_right {a b c : ℕ} (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c

-- artifact: Nat.Coprime Divides Cancellation on ℕ repair [pending]
-- artifact id: artifact_branch_4764c2dd-6914-4313-9b0d-7f328ea61e91
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_4764c2dd-6914-4313-9b0d-7f328ea61e91
-- label: Nat.Coprime Divides Cancellation on ℕ repair
-- metadata: {"hidden":true}

theorem coprime_dvd_of_dvd_mul_right {a b c : ℕ}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  apply Nat.Coprime.dvd_of_dvd_mul_right hcop
  simpa [Nat.mul_comm] using hdiv