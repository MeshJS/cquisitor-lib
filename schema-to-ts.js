#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/**
 * Convert JSON Schema type to TypeScript type
 */
function jsonSchemaTypeToTSType(schema, defsMap = new Map()) {
    if (!schema) return 'any';
    
    // Handle references
    if (schema.$ref) {
        const refName = schema.$ref.replace('#/$defs/', '').replace('#/definitions/', '');
        return refName;
    }
    
    // Handle arrays
    if (schema.type === 'array') {
        const itemType = jsonSchemaTypeToTSType(schema.items, defsMap);
        return `${itemType}[]`;
    }
    
    // Handle objects
    if (schema.type === 'object') {
        if (schema.properties) {
            const props = Object.entries(schema.properties).map(([key, prop]) => {
                const isRequired = schema.required && schema.required.includes(key);
                const propType = jsonSchemaTypeToTSType(prop, defsMap);
                return `  ${key}${isRequired ? '' : '?'}: ${propType};`;
            }).join('\n');
            return `{\n${props}\n}`;
        }
        return 'Record<string, any>';
    }
    
    // Handle enums
    if (schema.enum) {
        return schema.enum.map(val => `"${val}"`).join(' | ');
    }
    
    // Handle oneOf (union types)
    if (schema.oneOf) {
        return schema.oneOf.map(s => jsonSchemaTypeToTSType(s, defsMap)).join(' | ');
    }
    
    // Handle anyOf (union types)
    if (schema.anyOf) {
        return schema.anyOf.map(s => jsonSchemaTypeToTSType(s, defsMap)).join(' | ');
    }
    
    // Handle allOf (intersection types)
    if (schema.allOf) {
        return schema.allOf.map(s => jsonSchemaTypeToTSType(s, defsMap)).join(' & ');
    }
    
    // Handle basic types
    switch (schema.type) {
        case 'string':
            return 'string';
        case 'number':
        case 'integer':
            return 'number';
        case 'boolean':
            return 'boolean';
        case 'null':
            return 'null';
        default:
            return 'any';
    }
}

/**
 * Generate .d.ts style function declarations for parsing/serialization
 */
function generateDtsDeclarations(name) {
    return `
/**
 * Parse ${name} from JSON string
 */
declare function parse${name}(jsonString: string): ${name};

/**
 * Validate and parse ${name} from JSON string with runtime checking
 */
declare function parseSafe${name}(jsonString: string): ${name};

/**
 * Serialize ${name} to JSON string
 */
declare function stringify${name}(data: ${name}): string;

`;
}

/**
 * Generate all types in a single .d.ts file without duplication
 */
function generateDtsFile(allSchemas, outputPath) {
    // Collect all type definitions from all schemas
    const allTypeDefs = new Map();
    const mainTypes = [];
    
    // First pass: collect all type definitions and identify main types
    Object.entries(allSchemas).forEach(([mainTypeName, schema]) => {
        mainTypes.push(mainTypeName);
        
        // Add main type
        allTypeDefs.set(mainTypeName, schema);
        
        // Add all definitions from this schema
        const defs = schema.$defs || schema.definitions || {};
        Object.entries(defs).forEach(([defName, defSchema]) => {
            // Only add if not already defined (avoid duplicates)
            if (!allTypeDefs.has(defName)) {
                allTypeDefs.set(defName, defSchema);
            }
        });
    });
    
    console.log(`📝 Generating .d.ts file with ${allTypeDefs.size} total types`);
    
    let content = `// Auto-generated TypeScript type declarations from JSON schemas
// Generated at: ${new Date().toISOString()}
//
// This file contains all type declarations in .d.ts style without implementation.

`;
    
    // Generate all type definitions (excluding main types for now)
    allTypeDefs.forEach((defSchema, defName) => {
        if (!mainTypes.includes(defName)) {
            const tsType = jsonSchemaTypeToTSType(defSchema, allTypeDefs);
            if (defSchema.type === 'object') {
                content += `declare interface ${defName} ${tsType}\n\n`;
            } else {
                content += `declare type ${defName} = ${tsType};\n\n`;
            }
        }
    });
    
    // Generate main types
    mainTypes.forEach(typeName => {
        const schema = allSchemas[typeName];
        const tsType = jsonSchemaTypeToTSType(schema, allTypeDefs);
        if (schema.type === 'object') {
            content += `declare interface ${typeName} ${tsType}\n\n`;
        } else {
            content += `declare type ${typeName} = ${tsType};\n\n`;
        }
    });
    
    // Generate function declarations for each main type
    mainTypes.forEach(typeName => {
        content += generateDtsDeclarations(typeName);
    });
    
    return content;
}

/**
 * Main function
 */
function main() {
    const args = process.argv.slice(2);
    const schemasDir = args[0] || 'schemas';
    const outputDir = args[1] || 'types';
    
    console.log(`Converting JSON schemas from '${schemasDir}' to TypeScript .d.ts in '${outputDir}'`);
    
    // Create output directory
    if (!fs.existsSync(outputDir)) {
        fs.mkdirSync(outputDir, { recursive: true });
    }
    
    // Schema files to process
    const schemaFiles = [
        'NecessaryInputData.schema.json',
        'ValidationResult.schema.json',
        'ValidationInputContext.schema.json'
    ];
    
    // Load all schemas
    const allSchemas = {};
    schemaFiles.forEach(filename => {
        const schemaPath = path.join(schemasDir, filename);
        if (fs.existsSync(schemaPath)) {
            try {
                const schemaContent = fs.readFileSync(schemaPath, 'utf8');
                const schema = JSON.parse(schemaContent);
                const typeName = filename.replace('.schema.json', '');
                allSchemas[typeName] = schema;
                console.log(`📋 Loaded schema for ${typeName}`);
            } catch (error) {
                console.warn(`⚠️  Failed to load schema ${filename}: ${error.message}`);
            }
        } else {
            console.warn(`⚠️  Schema file not found: ${schemaPath}`);
        }
    });
    
    if (Object.keys(allSchemas).length === 0) {
        console.error('❌ No schemas loaded. Exiting.');
        return;
    }
    
    // Generate single combined .d.ts file
    const dtsFilePath = path.join(outputDir, 'index.d.ts');
    const dtsFileContent = generateDtsFile(allSchemas, dtsFilePath);
    
    fs.writeFileSync(dtsFilePath, dtsFileContent);
    console.log(`✅ Generated .d.ts file: ${dtsFilePath}`);
    
    console.log('\n🎉 TypeScript declaration file generation completed!');
    console.log(`📁 Generated file: ${dtsFilePath}`);
    console.log('\nUsage example:');
    console.log("  /// <reference path=\"./types/index.d.ts\" />");
}

// Run the script
main(); 