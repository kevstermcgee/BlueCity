import array,math,wave,random,json,hashlib,pathlib
root=pathlib.Path(__file__).resolve().parents[1] / 'assets/audio/city-ambience'; rate=44100;seconds=12;n=rate*seconds;rng=random.Random(7547)
# Authored procedural city bed: distant road rumble, wind, birds, and a crossing bell.
for name in ['day','night','rain']:
    values=array.array('h');sm=0.;prev=0.
    for i in range(n):
        t=i/rate;sm=sm*.992+(rng.random()*2-1)*.008
        wind=sm*.4;rumble=math.sin(math.tau*58*t)*.008*(.5+.5*math.sin(math.tau*t/seconds))
        bird=0.
        if name=='day':
            for at in [1.1,3.7,7.3,10.1]:
                u=t-at
                if 0<u<.24:bird+=math.sin(math.tau*(1800*u+1500*u*u))*.02*math.sin(math.pi*u/.24)**2
        rain=(rng.random()*2-1)*.025 if name=='rain' else 0
        sample=(wind+rumble+bird+rain)*(math.sin(math.pi*min(1,t/.08))**2 if t<.04 else 1)
        # Short common endpoint fade keeps loop seam quiet.
        fade=min(1,t/.05,(seconds-t)/.05)
        values.append(int(max(-.8,min(.8,sample*fade))*32767))
    path=root/f'{name}.wav'
    with wave.open(str(path),'wb') as w:w.setnchannels(1);w.setsampwidth(2);w.setframerate(rate);w.writeframes(values.tobytes())
meta={"version":1,"sample_rate":rate,"effects":{},"music":{},"music_scale":1.}
for name in ['day','night','rain']:
    path=root/f'{name}.wav';b=path.read_bytes();h=0xcbf29ce484222325
    for byte in b:h=((h^byte)*0x100000001b3)&((1<<64)-1)
    with wave.open(str(path))as w:v=array.array('h',w.readframes(n))
    v=[x/32768 for x in v]
    meta['music'][name]={"file":path.name,"checksum":f'{h:016x}',"frames":n,"peak":max(map(abs,v)),"rms":math.sqrt(sum(x*x for x in v)/n),"dc":abs(sum(v)/n),"seam_jump":abs(v[0]-v[-1])}
(root/'bank.json').write_text(json.dumps(meta,indent=2)+'\n')
