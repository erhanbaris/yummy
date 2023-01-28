# Lua Scripting

**Yummy** has *Lua Programming Language (v5.4)* support. You can control the flow via Lua scripts and it is very easy to write. All **Auth** module can be controlled over Lua and the system has default Lua script to start modification. **Yummy** will have more robust Lua support but for now, we have limited support. Also, all Lua scripts are stateful, it means that if you change variable at runtime, it will change that variable for all other requests. 

**Yummy** scan all available Lua files and import into the system. You should configure `DEFAULT_LUA_FILES_PATH` parameter to change default path.
You should check `server/lua/auth.lua` files see all available API's.

### WARNING
There is a one critical issue can be impect your Lua implementation.

**Never save/assign *Model* to lua list/table/variable.** That will impect **Yummy** stability and the system will **CRASH** at first request. But, nice thing is that you will get information about why it is crashed and clearly, you can read that problem at logs.

Here is the one the example for **DO NOT DO THIS**.

```lua
messages = {}

function pre_email_auth(model)
    messages.pre_auth = model
end

function post_email_auth(model, successed)
    messages.post_auth = model
end
```

### TODOS

- [ ] Database access
- [ ] Stateless access
- [ ] Unit tests
