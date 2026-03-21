import Mathlib
/-!
OpenProof scratch session: Nat Coprime Divisibility Cancellation
Session id: session_27cae29b-374c-4273-8b6a-ec340abdcd2a
Mode: research
Updated: 
-/
-- branch: Nat Coprime Divisibility Cancellation prover [proving]
-- branch id: branch_02d7c8a8-5a6e-4ed9-bf07-8dd098c93282
-- node: Nat Coprime Divisibility Cancellation [proving]
-- kind: theorem
-- statement: theorem nat_coprime_dvd_of_dvd_mul_right {a b c : ℕ} (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c

-- artifact: Nat Coprime Divisibility Cancellation prover [pending]
-- artifact id: artifact_branch_02d7c8a8-5a6e-4ed9-bf07-8dd098c93282
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_02d7c8a8-5a6e-4ed9-bf07-8dd098c93282
-- label: Nat Coprime Divisibility Cancellation prover
-- metadata: {"hidden":false,"foreground":true}

theorem nat_coprime_dvd_of_dvd_mul_right {a b c : ℕ}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  exact hcop.dvd_of_dvd_mul_right hdiv