import json, sys
S = sys.argv[1]
d = json.load(open(S + '/ref/q01-2d-plate.camproj'))
E = {e['name']: e for e in d['entities']}
def tooldef(n):
    t = dict(E[n]['tool']); t.pop('holder', None); return t
def opp(n):
    p = E[n]['operation']['params']; return list(p.values())[0]
C = []
def c(step, _t, **a): C.append({"step": step, "name": _t, "arguments": a})
c('2','import_model',path='{RUN}/q01-2d-plate.step',name='q01-2d-plate')
c('2','load_machine',builtin='Tormach 770MX')
c('2','set_work_material',material='aluminium_6061')
c('2','fit_stock',sides=[3,3,3,3,0,1])
c('2','add_setup',name='Setup1')
c('2','set_wcs',stock_point={"x":"min","y":"min","z":"max"})
c('2','set_setup_post',post='Tormach PathPilot')
c('2','set_stock_material',material='aluminium_6061')
c('3','add_tool',catalog='EM-1000-4FL')
c('3','add_tool',catalog='EM-0600-4FL')
for n in ['SPOT-0800-90','DR-0500','DR-0660']:
    c('3','add_tool',name=n,definition=tooldef(n))
c('3','add_tool',catalog='TAP-M6')
c('3','add_tool',catalog='CM-1000-90')
c('4f','list_features',model='q01-2d-plate',kinds=['chains'],compact=True)
c('4f','detect_pockets',model='q01-2d-plate')
c('4a','add_operation',kind='face',name='Face_Top',source='stock',tool='EM-1000-4FL',params=opp('Face_Top'))
c('4b','detect_pockets',model='q01-2d-plate',create=1,tool='EM-0600-4FL',name='Pocket_Center')
pp = opp('Pocket_Center'); pp = {k: pp[k] for k in ['stepover','stepdown','feed','plunge_feed','entry','helix_radius','strategy','climb']}
c('4b','edit_operation',name='Pocket_Center',params=pp)
for nm, tool in [('Spot_All_5','SPOT-0800-90'),('Drill_6_6_Thru','DR-0660'),('Drill_5_0_Blind','DR-0500'),('Tap_M6','TAP-M6')]:
    c('4d','add_operation',kind='drill',name=nm,source='q01-2d-plate',tool=tool)
    c('4d','detect_holes',name=nm)
    c('4d','edit_operation',name=nm,params=opp(nm))
c('4c','add_operation',kind='contour',name='Contour_Outside',source='q01-2d-plate',tool='EM-1000-4FL',params=opp('Contour_Outside'))
c('4c','set_operation_geometry',operation='Contour_Outside',picks=[{"model":"q01-2d-plate","id":"chain:0x65b60046ba17817d"}])
c('4e','add_operation',kind='chamfer2_d',name='Chamfer_Outside_Top',source='q01-2d-plate',tool='CM-1000-90',params=opp('Chamfer_Outside_Top'))
c('4e','set_operation_geometry',operation='Chamfer_Outside_Top',picks=[{"model":"q01-2d-plate","id":"chain:0x60e930a9e55bf92d"}])
c('5','generate')
c('5','verify')
c('5','render_stock',view='iso',width=64,height=48)
c('5','post',output='{RUN}/q01-2d-plate.nc',post='Tormach PathPilot')
c('5','save_project',path='{RUN}/q01-2d-plate.camproj')
c('5','get_job',summary=True)
c('X','post',output='{RUN}/sp dir ü/q01 plate ü.nc',post='Tormach PathPilot')
c('X','save_project',path='{RUN}/sp dir ü/q01 plate ü.camproj')
c('X','open_project',path='{RUN}/sp dir ü/q01 plate ü.camproj')
c('X','import_model',path='{RUN}/sp dir ü/q01 plate ü.step',name='uni')
c('X','delete_model',name='uni')
c('X','generate')
c('X','verify')
c('X','post',output='{RUN}/sp dir ü/q01 plate ü-reopen.nc',post='Tormach PathPilot')
open(S + '/drv/calls_flow.jsonl','w',encoding='utf-8').write("\n".join(json.dumps(x, ensure_ascii=False) for x in C) + "\n")
R = []
def r(_t, **a): R.append({"step":"R","name":_t,"arguments":a})
r('open_project', path='{RUN}/ref/q01-2d-plate.camproj')
r('verify')
r('post', output='{RUN}/ref/ref-asis.nc', post='Tormach PathPilot')
r('generate'); r('verify')
r('post', output='{RUN}/ref/ref-regen.nc', post='Tormach PathPilot')
r('save_project', path='{RUN}/ref/ref-resaved.camproj')
open(S + '/drv/calls_ref.jsonl','w',encoding='utf-8').write("\n".join(json.dumps(x, ensure_ascii=False) for x in R) + "\n")
