#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Starting TypeScript Declaration (.d.ts) Generation Pipeline${NC}"
echo "=================================================="

# Step 1: Generate JSON schemas from Rust
echo -e "\n${YELLOW}📋 Step 1: Generating JSON schemas from Rust...${NC}"
if cargo run --bin generate_schemas; then
    echo -e "${GREEN}✅ JSON schemas generated successfully${NC}"
else
    echo -e "${RED}❌ Failed to generate JSON schemas${NC}"
    exit 1
fi

# Step 2: Convert schemas to TypeScript declarations
echo -e "\n${YELLOW}🔄 Step 2: Converting schemas to TypeScript declarations...${NC}"
if node schema-to-ts.js schemas types; then
    echo -e "${GREEN}✅ TypeScript declarations generated successfully${NC}"
else
    echo -e "${RED}❌ Failed to generate TypeScript declarations${NC}"
    exit 1
fi

# Step 3: Show results
echo -e "\n${GREEN}🎉 Pipeline completed successfully!${NC}"
echo -e "${BLUE}📁 Generated files:${NC}"
echo "  - schemas/NecessaryInputData.schema.json"
echo "  - schemas/ValidationResult.schema.json" 
echo "  - schemas/ValidationInputContext.schema.json"
echo "  - types/index.d.ts"

echo -e "\n${BLUE}📖 Usage:${NC}"
echo "  // Reference the declaration file:"
echo "  /// <reference path=\"./types/index.d.ts\" />"

echo -e "\n${GREEN}✨ TypeScript declaration files are ready to use!${NC}" 