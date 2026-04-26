# Biome Quick Reference

## Common Commands

```bash
# Check for issues
npx @biomejs/biome check .

# Format code
npx @biomejs/biome format --write .

# Check and apply safe fixes
npx @biomejs/biome check --write .

# Check specific files
npx @biomejs/biome check src/components/**/*.tsx

# Show more diagnostics
npx @biomejs/biome check --max-diagnostics=500 .
```

## Suppression Comments

```typescript
// Ignore specific rule
// biome-ignore lint/suspicious/noArrayIndexKey: Static list with stable order
const items = array.map((item, index) => <div key={index}>{item}</div>);

// Ignore entire file
// biome-ignore-all lint: Legacy code, will refactor later

// Ignore multiple rules
// biome-ignore lint/suspicious/noExplicitAny lint/correctness/noUnusedVariables: Type definition file
```

## Configuration Tips

### Exclude Files
```json
{
  "files": {
    "includes": [
      "src/**/*",
      "!node_modules",
      "!dist",
      "!*.config.js"
    ]
  }
}
```

### Disable Specific Rules
```json
{
  "linter": {
    "rules": {
      "suspicious": {
        "noArrayIndexKey": "off"
      }
    }
  }
}
```

## Current Project Status (2026-04-26)

- **Total Issues:** 105 (92 errors, 13 warnings)
- **Main Categories:**
  - 24 noArrayIndexKey (static lists)
  - 17 useExhaustiveDependencies (React hooks)
  - 13 noExplicitAny (polyfills)
  - 43 accessibility warnings

## When to Use Suppressions

✅ **Good reasons:**
- Static lists with stable order (noArrayIndexKey)
- Intentional any types in polyfills
- Legacy code with refactor plan

❌ **Bad reasons:**
- "It's too hard to fix"
- Silencing real bugs
- Avoiding proper type definitions

## Integration with CI/CD

```bash
# Add to package.json scripts
"lint": "biome check .",
"lint:fix": "biome check --write .",
"format": "biome format --write .",
"format:check": "biome format ."
```

## Migration from Prettier/ESLint

✅ **Already done:**
- Formatter config matches Prettier
- Most ESLint rules covered
- Auto-formatting working

⚠️ **Note:** Some ESLint plugins (like import sorting) need separate handling
