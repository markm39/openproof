import Mathlib
/-!
OpenProof scratch session: Nat Coprime Divisibility Cancellation
Session id: session_27cae29b-374c-4273-8b6a-ec340abdcd2a
Mode: research
Updated: 
-/
-- branch: Nat Coprime Divisibility Cancellation planner [proving]
-- branch id: branch_43010834-a753-4d3a-86b0-67668ea4a467
-- node: Nat Coprime Divisibility Cancellation [proving]
-- kind: theorem
-- statement: theorem nat_coprime_dvd_of_dvd_mul_right {a b c : ℕ} (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c

-- artifact: Nat Coprime Divisibility Cancellation planner [pending]
-- artifact id: artifact_branch_43010834-a753-4d3a-86b0-67668ea4a467
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_43010834-a753-4d3a-86b0-67668ea4a467
-- label: Nat Coprime Divisibility Cancellation planner
-- metadata: {"hidden":true}

theorem nat_coprime_dvd_of_dvd_mul_right {a b c : ℕ}
    (hcop : Nat.Coprime a b) (hdiv : a ∣ b * c) : a ∣ c := by
  apply hcop.dvd_of_dvd_mul_right
  simpa [Nat.mul_comm] using hdiv