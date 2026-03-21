import Mathlib
/-!
OpenProof scratch session: Divisibility Closed Under Addition
Session id: session_bbb6890a-6ebe-46fb-9912-38aa5d5b500b
Mode: research
Updated: 
-/
-- branch: Divisibility Closed Under Addition prover [proving]
-- branch id: branch_f35e00cc-2eb8-47e9-b826-942c0ee214cb
-- node: Divisibility Closed Under Addition [proving]
-- kind: theorem
-- statement: `theorem gcd_dvd_left' (m n : ℕ) : Nat.gcd m n ∣ m`

-- artifact: Divisibility Closed Under Addition prover [pending]
-- artifact id: artifact_branch_f35e00cc-2eb8-47e9-b826-942c0ee214cb
-- artifact kind: snippet

-- target kind: artifact
-- target id: artifact_branch_f35e00cc-2eb8-47e9-b826-942c0ee214cb
-- label: Divisibility Closed Under Addition prover
-- metadata: {"foreground":true}

theorem gcd_dvd_left' (m n : ℕ) : Nat.gcd m n ∣ m :=
  Nat.gcd_dvd_left m n