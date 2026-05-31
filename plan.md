1. Modify `src/semantic/conversion.rs` using `run_in_bash_session` to add type validation in `classify_insert`, `classify_push`, and `classify_pop`.
   - The implementation was already verified during exploration.
   - Execute the following exact command:
```bash
cat << 'EOF' > fix.patch
--- src/semantic/conversion.rs	2024-05-31 15:45:00.000000000 +0000
+++ src/semantic/conversion.rs	2024-05-31 16:00:00.000000000 +0000
@@ -578,11 +578,21 @@
         return Ok(None);
     };

+    let subj_name = &subject.normalized;
+    let subj_type = scope
+        .lookup(subj_name)
+        .cloned()
+        .unwrap_or(GlossaType::Unknown);
+
+    if matches!(subj_type, GlossaType::Number | GlossaType::String | GlossaType::Boolean) {
+        return Err(GlossaError::semantic(format!(
+            "Τὸ «{}» ἔστιν {}, οὐχὶ συλλογή. Οὐ δύναται ἕλκειν.",
+            subj_name, subj_type.to_greek()
+        )));
+    }
+
     let receiver = AnalyzedExpr {
-        expr: AnalyzedExprKind::Variable(subject.lemma.clone()),
-        glossa_type: scope
-            .lookup(&subject.lemma)
-            .cloned()
-            .unwrap_or(GlossaType::Unknown),
+        expr: AnalyzedExprKind::Variable(subj_name.clone()),
+        glossa_type: subj_type.clone(),
     };

@@ -604,11 +614,21 @@
         return Ok(None);
     };

+    let subj_name = &subject.normalized;
+    let subj_type = scope
+        .lookup(subj_name)
+        .cloned()
+        .unwrap_or(GlossaType::Unknown);
+
+    if matches!(subj_type, GlossaType::Number | GlossaType::String | GlossaType::Boolean) {
+        return Err(GlossaError::semantic(format!(
+            "Τὸ «{}» ἔστιν {}, οὐχὶ συλλογή. Οὐ δύναται ὠθεῖν.",
+            subj_name, subj_type.to_greek()
+        )));
+    }
+
     let receiver = AnalyzedExpr {
-        expr: AnalyzedExprKind::Variable(subject.lemma.clone()),
-        glossa_type: scope
-            .lookup(&subject.lemma)
-            .cloned()
-            .unwrap_or(GlossaType::Unknown),
+        expr: AnalyzedExprKind::Variable(subj_name.clone()),
+        glossa_type: subj_type.clone(),
     };

@@ -662,6 +682,13 @@
         .cloned()
         .unwrap_or(GlossaType::Unknown);

+    if matches!(subj_type, GlossaType::Number | GlossaType::String | GlossaType::Boolean) {
+        return Err(GlossaError::semantic(format!(
+            "Τὸ «{}» ἔστιν {}, οὐχὶ συλλογή. Οὐ δύναται τιθέναι.",
+            subj_name, subj_type.to_greek()
+        )));
+    }
+
     let receiver = AnalyzedExpr {
         expr: AnalyzedExprKind::Variable(subj_name.clone()),
         glossa_type: subj_type.clone(),
