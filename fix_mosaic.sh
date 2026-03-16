#!/bin/bash
cat << 'INNER_EOF' > /tmp/mosaic_patch.diff
--- a/src/tools/mosaic.rs
+++ b/src/tools/mosaic.rs
@@ -140,12 +140,13 @@
         .unwrap_or_default();

     // Combine nominatives if subject is present or if there are multiple
-    let extra_noms = asm
-        .nominatives
-        .iter()
-        .map(fmt_constituent)
-        .collect::<Vec<_>>()
-        .join(", ");
+    let mut extra_noms = String::with_capacity(asm.nominatives.len() * 16);
+    for (i, nom) in asm.nominatives.iter().enumerate() {
+        if i > 0 {
+            extra_noms.push_str(", ");
+        }
+        extra_noms.push_str(&fmt_constituent(nom));
+    }
     let full_subject = if !extra_noms.is_empty() {
         if !subject.is_empty() {
             format!("{} (+ {})", subject, extra_noms)
@@ -183,34 +184,37 @@

     // Genitives
     if !asm.genitives.is_empty() {
-        let gens = asm
-            .genitives
-            .iter()
-            .map(fmt_constituent)
-            .collect::<Vec<_>>()
-            .join(", ");
+        let mut gens = String::with_capacity(asm.genitives.len() * 16);
+        for (i, g) in asm.genitives.iter().enumerate() {
+            if i > 0 {
+                gens.push_str(", ");
+            }
+            gens.push_str(&fmt_constituent(g));
+        }
         other.push(format!("Gen: [{}]", gens));
     }

     // Adjectives
     if !asm.adjectives.is_empty() {
-        let adjs = asm
-            .adjectives
-            .iter()
-            .map(fmt_constituent)
-            .collect::<Vec<_>>()
-            .join(", ");
+        let mut adjs = String::with_capacity(asm.adjectives.len() * 16);
+        for (i, a) in asm.adjectives.iter().enumerate() {
+            if i > 0 {
+                adjs.push_str(", ");
+            }
+            adjs.push_str(&fmt_constituent(a));
+        }
         other.push(format!("Adj: [{}]", adjs));
     }

     // Participles
     if !asm.participles.is_empty() {
-        let parts = asm
-            .participles
-            .iter()
-            .map(|p| p.original.to_string())
-            .collect::<Vec<_>>()
-            .join(", ");
+        let mut parts = String::with_capacity(asm.participles.len() * 16);
+        for (i, p) in asm.participles.iter().enumerate() {
+            if i > 0 {
+                parts.push_str(", ");
+            }
+            parts.push_str(&p.original);
+        }
         other.push(format!("Participles: [{}]", parts));
     }

@@ -222,12 +226,13 @@
         other.push(format!("Index Accesses: {}", asm.index_accesses.len()));
     }
     if !asm.property_accesses.is_empty() {
-        let props = asm
-            .property_accesses
-            .iter()
-            .map(|(o, p)| format!("{}.{}", o, p))
-            .collect::<Vec<_>>()
-            .join(", ");
+        let mut props = String::with_capacity(asm.property_accesses.len() * 16);
+        for (i, (o, p)) in asm.property_accesses.iter().enumerate() {
+            if i > 0 {
+                props.push_str(", ");
+            }
+            props.push_str(&format!("{}.{}", o, p));
+        }
         other.push(format!("Properties: [{}]", props));
     }
     if !asm.blocks.is_empty() {
INNER_EOF
patch -p1 < /tmp/mosaic_patch.diff
