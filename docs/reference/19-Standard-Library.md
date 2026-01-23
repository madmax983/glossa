# 19. Standard Library

## 19.1 Core Verbs

| Greek | Lemma | Meaning | Operation |
|-------|-------|---------|-----------|
| λέγω | λεγω | say, speak | print |
| γράφω | γραφω | write | write, serialize |
| εἰμί | ειμι | be | binding (ἔστω), equality |
| ἔχω | εχω | have | contains, has |
| δίδωμι | διδωμι | give | return, yield |
| λαμβάνω | λαμβανω | take | receive, get |
| ὠθέω | ὠθεω | push | push to collection |
| ἕλκω | ἑλκω | pull | pop from collection |
| εὑρίσκω | εὑρισκω | find | find, search |
| τίθημι | τιθημι | place | set, insert |
| ποιέω | ποιεω | make | create, construct |
| φέρω | φερω | carry | move, transfer |
| παύω | παυω | stop | break |
| συνεχίζω | συνεχιζω | continue | continue |

## 19.2 Core Nouns

| Greek | Gender | Meaning | Type |
|-------|--------|---------|------|
| ἀριθμός | M | number | i64 |
| ῥητόν | N | rational/float | f64 |
| λόγος | M | word, string | String |
| χαρακτήρ | M | character | char |
| ἀληθότυπος | M | boolean | bool |
| πίναξ | M | table, array | Vec<T> |
| λίστη | F | list | Vec<T> |
| χάρτης | M | map | HashMap |
| σύνολον | N | set | HashSet |
| ζεῦγος | N | pair | (T, U) |
| ὄνομα | N | name | identifier |
| τιμή | F | value, price | generic value |
| κλείς | F | key | key type |
| στοιχεῖον | N | element | element type |

## 19.3 Comparison Adjectives

| Greek | Meaning | Operator |
|-------|---------|----------|
| μέγας, μεῖζον | great, greater | > |
| μικρός, ἔλαττον | small, lesser | < |
| ἴσος, ἴσον | equal | == |
| πρῶτος | first | [0] |
| ἔσχατος | last | [-1] |
| κενός | empty | is_empty() |
| πλήρης | full | is_full() |

## 19.4 Particles

| Greek | Meaning | Use |
|-------|---------|-----|
| εἰ | if | conditional |
| ἐάν | if (+ subj) | conditional |
| εἰ δὲ μή | else | else clause |
| ἕως | while, until | while loop |
| μέχρι | until | range end |
| ἀπό | from | range start |
| διά | through | for loop |
| κατά | according to | match, by |
| καί | and | && |
| ἤ | or | \|\| |
| οὐ, οὐκ, οὐχ | not | ! |
| μή | not (non-indic) | ! in conditions |