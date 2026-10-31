// Focused, caller-seeded analysis. Original executable bytes are never changed.
// @category TFM2.RealmInvestigation
import ghidra.app.script.GhidraScript;
import ghidra.app.cmd.disassemble.DisassembleCommand;
import ghidra.app.decompiler.*;
import ghidra.program.model.address.*;
import ghidra.program.model.listing.*;
import ghidra.program.model.data.*;
import ghidra.program.model.symbol.*;
import com.google.gson.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;

public class RealmSeedAnalysis extends GhidraScript {
    private final Set<String> sdkPrototypes=new HashSet<>();
    private void prototype(String rva,String name,DataType result,String... names) throws Exception {
        Function f=getFunctionAt(currentProgram.getImageBase().add(Long.decode(rva)));
        if(f==null)return;
        f.setName(name,SourceType.USER_DEFINED);
        f.setCallingConvention("__fastcall");
        f.setReturnType(result,SourceType.USER_DEFINED);
        List<Variable> parameters=new ArrayList<>();
        for(String n:names) {
            DataType type=n.endsWith("_u32")?UnsignedIntegerDataType.dataType:
                n.equals("state")||n.endsWith("_pointer")?new PointerDataType(VoidDataType.dataType,8):UnsignedLongLongDataType.dataType;
            parameters.add(new ParameterImpl(n,type,currentProgram));
        }
        f.replaceParameters(parameters,Function.FunctionUpdateType.DYNAMIC_STORAGE_ALL_PARAMS,true,SourceType.USER_DEFINED);
        f.setComment("Caller-supplied installed SDK C ABI. Integer-sized handles represented as uint64. Private downstream ABI is not established.");
        sdkPrototypes.add(rva);
    }
    @Override public void run() throws Exception {
        String[] args=getScriptArgs();
        if(args.length!=2)throw new IllegalArgumentException("realm-hooks.json output.json");
        JsonObject input=JsonParser.parseString(Files.readString(Path.of(args[0]))).getAsJsonObject();
        String expected=input.get("exeSha256").getAsString();
        String actual=currentProgram.getExecutableSHA256();
        if(actual==null||!actual.equalsIgnoreCase(expected))throw new IllegalStateException("Imported executable digest mismatch");
        JsonObject report=new JsonObject();report.addProperty("version",1);report.addProperty("exeSha256",actual);
        report.addProperty("provider","Ghidra 12.1.4 focused headless analysis");
        report.addProperty("analysisCoverage","No global auto-analysis; instruction/function seeds are caller assertions from decoded PE bytes. Pseudocode is an analysis hypothesis, not original source.");
        report.addProperty("nativeBackendEnabled",false);
        Map<String,JsonObject> seeds=new LinkedHashMap<>();
        for(JsonElement element:input.getAsJsonArray("functions")) {
            JsonObject seed=element.getAsJsonObject();seeds.put(seed.get("rva").getAsString(),seed);
        }
        // Define all decoded seeds first, so direct calls have declared callees.
        Map<String,String> errors=new HashMap<>();
        for(Map.Entry<String,JsonObject> item:seeds.entrySet()) {
            String rva=item.getKey();JsonObject seed=item.getValue();
            Address entry=currentProgram.getImageBase().add(Long.decode(rva));
            AddressSet body=new AddressSet();
            for(JsonElement element:seed.getAsJsonArray("instructions")) {
                JsonObject instruction=element.getAsJsonObject();Address at=currentProgram.getImageBase().add(Long.decode(instruction.get("rva").getAsString()));
                int length=instruction.get("bytes").getAsString().length()/2;
                body.addRange(at,at.add(length-1));
            }
            if(body.isEmpty())continue;
            try {
                new DisassembleCommand(entry,body,true).applyTo(currentProgram,monitor);
                Function existing=getFunctionAt(entry);
                if(existing==null)currentProgram.getFunctionManager().createFunction("realm_seed_"+rva.substring(2),entry,body,SourceType.USER_DEFINED);
            }catch(Exception error){errors.put(rva,error.toString());}
        }
        // Canonical public signatures from installed mod-api-stable/src/host.rs.
        // This improves wrapper pseudocode, but does not claim private Rust ABI recovery.
        prototype("0x33cb210","sdk_is_visible",BooleanDataType.dataType,"state","team","entity_id");
        prototype("0x33cb980","sdk_deal_damage",VoidDataType.dataType,"state","attacker","target","ad","ap","attack_type_u32");
        prototype("0x33cd200","sdk_entity_set_pos",BooleanDataType.dataType,"state","handle","x","y");
        prototype("0x33cfd80","sdk_entity_set_invisible",BooleanDataType.dataType,"state","handle","ticks");
        prototype("0x33d0240","sdk_heal",VoidDataType.dataType,"state","caster","target","amount");
        prototype("0x33cabd0","sdk_apply_cc",VoidDataType.dataType,"state","target","cc_pointer");
        prototype("0x33cebb0","sdk_entity_add_shield",BooleanDataType.dataType,"state","handle","amount","duration_ticks");
        DecompInterface decompiler=new DecompInterface();
        decompiler.toggleCCode(true);decompiler.toggleSyntaxTree(true);
        decompiler.openProgram(currentProgram);
        String[] selected={"0x33cb210","0x16aa0c0","0x33d1140","0x33cd200","0x16ad6e0","0x33cfd80",
            "0x33cb980","0x33cabd0","0x33d0240","0x33cebb0","0x33ce6f0","0x33cb3e0","0x18eb590",
            "0x16ad0d0","0x33d2d60","0x1e130a0","0x170d240","0x33d0a10","0x18c0480",
            "0x1765840","0x1a098a0"};
        JsonArray functions=new JsonArray();
        try {
            for(String rva:selected) {
                monitor.checkCancelled();println("Decompiling selected seed "+rva);
                JsonObject row=new JsonObject();row.addProperty("rva",rva);row.addProperty("hookValidated",false);
                if(errors.containsKey(rva))row.addProperty("seedWarning",errors.get(rva));
                Address entry=currentProgram.getImageBase().add(Long.decode(rva));Function function=getFunctionAt(entry);
                if(function==null){row.addProperty("error","No function at exact requested entry");functions.add(row);continue;}
                row.addProperty("callingConvention",function.getCallingConventionName());
                row.addProperty("signatureSource",sdkPrototypes.contains(rva)?"caller-supplied installed SDK C ABI":"unverified decompiler inference");
                DecompileResults result=decompiler.decompileFunction(function,45,monitor);
                row.addProperty("decompileCompleted",result.decompileCompleted());
                row.addProperty("error",result.getErrorMessage());
                if(result.decompileCompleted()&&result.getDecompiledFunction()!=null)row.addProperty("pseudocode",result.getDecompiledFunction().getC());
                JsonArray references=new JsonArray();
                AddressIterator addresses=function.getBody().getAddresses(true);
                while(addresses.hasNext()) {
                    Address at=addresses.next();
                    for(Reference reference:getReferencesFrom(at)) {
                        if(reference.getReferenceType().isCall()) {
                            JsonObject edge=new JsonObject();edge.addProperty("from",at.toString());edge.addProperty("to",reference.getToAddress().toString());
                            edge.addProperty("type",reference.getReferenceType().toString());references.add(edge);
                        }
                    }
                }
                row.add("callReferences",references);functions.add(row);
            }
        } finally {decompiler.dispose();}
        report.add("functions",functions);
        Files.writeString(Path.of(args[1]),new GsonBuilder().setPrettyPrinting().create().toJson(report)+"\n",StandardCharsets.UTF_8);
        println("Focused realm analysis exported "+functions.size()+" selected functions");
    }
}
