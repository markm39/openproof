import Mathlib
/-!
OpenProof scratch session: Coprime Cancellation in a Natural-number Product
Session id: session_d3154de5-1fa6-45da-921a-bf1b1adc3de9
Mode: research
Updated: 
-/
-- branch: Coprime Cancellation in a Natural-number Product prover [proving]
-- branch id: branch_2c46b760-e0b9-4f36-bdca-e9683d46e997
-- node: Coprime Cancellation in a Natural-number Product [proving]
-- kind: theorem
-- statement: `theorem nat_coprime_dvd_of_dvd_mul {a b c : Nat} (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c`

-- artifact: Coprime Cancellation in a Natural-number Product prover [pending]
-- artifact id: artifact_branch_2c46b760-e0b9-4f36-bdca-e9683d46e997
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_2c46b760-e0b9-4f36-bdca-e9683d46e997
-- label: Coprime Cancellation in a Natural-number Product prover
-- metadata: {"hidden":false,"foreground":true}

theorem nat_coprime_dvd_of_dvd_mul {a b c : Nat}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  sorry