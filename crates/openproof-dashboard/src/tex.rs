//! LaTeX generation from proof session state.

use openproof_protocol::SessionSnapshot;

pub(crate) fn generate_tex(session: &SessionSnapshot) -> String {
    let proof = &session.proof;
    let title = &session.title;

    // If the model has written a LaTeX paper body, use it directly.
    if !proof.paper_tex.trim().is_empty() {
        // If paper_tex is already a complete document, return it as-is
        if proof.paper_tex.contains("\\documentclass") {
            return proof.paper_tex.clone();
        }
        let mut doc = String::new();
        doc.push_str("\\documentclass[11pt]{article}\n");
        doc.push_str("\\usepackage[margin=1in]{geometry}\n");
        doc.push_str("\\usepackage{fontspec}\n");
        doc.push_str("\\usepackage{amsmath,amssymb,amsthm}\n");
        doc.push_str("\\usepackage{listings}\n");
        doc.push_str("\\usepackage{xcolor}\n");
        doc.push_str("\\lstset{basicstyle=\\ttfamily\\small,breaklines=true,frame=single,backgroundcolor=\\color{gray!10},literate=\n");
        doc.push_str("  {ℕ}{{\\ensuremath{\\mathbb{N}}}}1\n");
        doc.push_str("  {ℝ}{{\\ensuremath{\\mathbb{R}}}}1\n");
        doc.push_str("  {ℤ}{{\\ensuremath{\\mathbb{Z}}}}1\n");
        doc.push_str("  {→}{{\\ensuremath{\\to}}}1\n");
        doc.push_str("  {←}{{\\ensuremath{\\leftarrow}}}1\n");
        doc.push_str("  {∀}{{\\ensuremath{\\forall}}}1\n");
        doc.push_str("  {∃}{{\\ensuremath{\\exists}}}1\n");
        doc.push_str("  {∧}{{\\ensuremath{\\land}}}1\n");
        doc.push_str("  {∨}{{\\ensuremath{\\lor}}}1\n");
        doc.push_str("  {≤}{{\\ensuremath{\\leq}}}1\n");
        doc.push_str("  {≥}{{\\ensuremath{\\geq}}}1\n");
        doc.push_str("  {≠}{{\\ensuremath{\\neq}}}1\n");
        doc.push_str("  {∈}{{\\ensuremath{\\in}}}1\n");
        doc.push_str("  {⟨}{{\\ensuremath{\\langle}}}1\n");
        doc.push_str("  {⟩}{{\\ensuremath{\\rangle}}}1\n");
        doc.push_str("  {λ}{{\\ensuremath{\\lambda}}}1\n");
        doc.push_str("  {∑}{{\\ensuremath{\\sum}}}1\n");
        doc.push_str("  {∞}{{\\ensuremath{\\infty}}}1\n");
        doc.push_str("}\n");
        doc.push_str("\\newtheorem{theorem}{Theorem}\n");
        doc.push_str("\\newtheorem{lemma}[theorem]{Lemma}\n");
        doc.push_str("\\newtheorem{proposition}[theorem]{Proposition}\n");
        doc.push_str(&format!("\n\\title{{{}}}\n", tex_escape(title)));
        doc.push_str("\\author{OpenProof}\n");
        doc.push_str("\\date{\\today}\n\n");
        doc.push_str("\\begin{document}\n\\maketitle\n\n");
        // Strip [language=Lean] etc. -- listings doesn't know Lean.
        let sanitized = proof
            .paper_tex
            .replace("[language=Lean]", "")
            .replace("[language=lean]", "")
            .replace("[language=lean4]", "")
            .replace("[language=Lean4]", "");
        doc.push_str(&sanitized);
        doc.push_str("\n\n\\end{document}\n");
        return doc;
    }

    // Fallback: mechanical generation from proof state.
    let mut doc = String::new();
    doc.push_str("\\documentclass[11pt]{article}\n");
    doc.push_str("\\usepackage[margin=1in]{geometry}\n");
    doc.push_str("\\usepackage{amsmath,amssymb,amsthm}\n");
    doc.push_str("\\usepackage{listings}\n");
    doc.push_str("\\usepackage{xcolor}\n");
    doc.push_str("\\lstset{basicstyle=\\ttfamily\\small,breaklines=true,frame=single,backgroundcolor=\\color{gray!10}}\n");
    doc.push_str("\\newtheorem{theorem}{Theorem}\n");
    doc.push_str("\\newtheorem{lemma}[theorem]{Lemma}\n");
    doc.push_str("\\newtheorem{proposition}[theorem]{Proposition}\n");
    doc.push('\n');
    doc.push_str(&format!("\\title{{{}}}\n", tex_escape(title)));
    doc.push_str("\\author{OpenProof}\n");
    doc.push_str("\\date{\\today}\n");
    doc.push_str("\n\\begin{document}\n\\maketitle\n\n");

    // Problem statement
    if let Some(problem) = &proof.problem {
        if !problem.trim().is_empty() {
            doc.push_str("\\section*{Problem}\n");
            doc.push_str(&tex_escape(problem));
            doc.push_str("\n\n");
        }
    }

    // Formal target
    if let Some(target) = &proof.formal_target {
        if !target.trim().is_empty() {
            doc.push_str("\\section*{Formal Target}\n");
            doc.push_str("\\begin{lstlisting}[language={}]\n");
            doc.push_str(target);
            doc.push_str("\n\\end{lstlisting}\n\n");
        }
    }

    // Proof nodes
    if !proof.nodes.is_empty() {
        doc.push_str("\\section{Proof Structure}\n\n");
        for node in &proof.nodes {
            let env = match node.kind {
                openproof_protocol::ProofNodeKind::Theorem => "theorem",
                openproof_protocol::ProofNodeKind::Lemma => "lemma",
                _ => "proposition",
            };
            let status_marker = match node.status {
                openproof_protocol::ProofNodeStatus::Verified => {
                    " \\textnormal{[\\textcolor{green!70!black}{verified}]}"
                }
                openproof_protocol::ProofNodeStatus::Failed => {
                    " \\textnormal{[\\textcolor{red}{failed}]}"
                }
                openproof_protocol::ProofNodeStatus::Proving => {
                    " \\textnormal{[\\textcolor{orange}{proving}]}"
                }
                _ => "",
            };
            doc.push_str(&format!(
                "\\begin{{{env}}}[{}]{status_marker}\n",
                tex_escape(&node.label)
            ));
            if !node.statement.is_empty() {
                doc.push_str(&tex_escape(&node.statement));
                doc.push('\n');
            }
            doc.push_str(&format!("\\end{{{env}}}\n\n"));

            if !node.content.trim().is_empty() {
                doc.push_str("\\begin{lstlisting}[language={}]\n");
                doc.push_str(&node.content);
                doc.push_str("\n\\end{lstlisting}\n\n");
            }
        }
    }

    // Paper notes
    if !proof.paper_notes.is_empty() {
        doc.push_str("\\section{Notes}\n\n");
        doc.push_str("\\begin{itemize}\n");
        for note in &proof.paper_notes {
            doc.push_str(&format!("\\item {}\n", tex_escape(note)));
        }
        doc.push_str("\\end{itemize}\n\n");
    }

    // Strategy summary
    if let Some(strategy) = &proof.strategy_summary {
        if !strategy.trim().is_empty() {
            doc.push_str("\\section{Strategy}\n\n");
            doc.push_str(&tex_escape(strategy));
            doc.push_str("\n\n");
        }
    }

    doc.push_str("\\end{document}\n");
    doc
}

pub(crate) fn tex_escape(s: &str) -> String {
    s.replace('\\', "\\textbackslash{}")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('&', "\\&")
        .replace('%', "\\%")
        .replace('$', "\\$")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('^', "\\^{}")
        .replace('~', "\\~{}")
}
