% test hrtf iir optimization
SOFAstart;
HRTF = SOFAload("Kemar_HRTF_sofa.sofa");

hrtfs = HRTF.Data.IR;
angles = HRTF.SourcePosition;
angles = angles(32401: 32400 + 360, :);
hrtfs = squeeze(hrtfs(32401: 32400 + 360,:,1:128));
hrtfs = permute(hrtfs,[3,2,1]);

% plot(squeeze(hrtfs(1,:,:))');

[~, hrtf_mp] = rceps(hrtf(1:100,:));

delays = [-50:0.5:50];
fracDel = dsp.VariableFractionalDelay('InterpolationMethod','FIR');

hrtf = squeeze(hrtfs(:,:,90));
hrtf_s = sum(hrtf,2);
[~, hrtf_mp] = rceps(hrtf); 
for k = 1:10 %length(delays)
    
    hrtf_mp(:,1) = fracDel(hrtf_mp(:,1), delays(k));%length(delays)
    hrtf_fft = 20*log10(abs(fft(hrtf_del)));
end