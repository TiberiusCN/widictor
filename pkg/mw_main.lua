package.path = arg[1] .. '/?.lua;'
   .. '/tmp/widictor/modules/?.lua'

require('MWServer')
require('mwInit')
server = MWServer:new( arg[2], arg[3] )
server:execute()

