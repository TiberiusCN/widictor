local export = {}

export.ustring = utf8
export.text = {}
export.title = {}

function export.title.getCurrentTitle()
  return os.getenv("ENV_MAINWORD")
end

function export.loadData(module)
   return require(module)
end 

function export.text.split(inputstr, sep)
   if sep == nil then
      sep = "%s"
   end
   local t={}
   for str in string.gmatch(inputstr, "([^"..sep.."]+)") do
      table.insert(t, str)
   end
   return t
end

return export
