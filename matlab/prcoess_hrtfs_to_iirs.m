% hrtf_to_ir
clear all
test = false; 

% load minphase hrtfs
load('KERMAR_HRIRs_minPhase.mat');
% load itds
% load('KEMAR_ITD_Delay_raw.mat');
load('KEMAR_AACHEN_ITD_s.mat');
% load angles
load('ANGLES_RAW.mat');
% load raw HRTFS
load('KEMAR_HRTF_RAW.mat');
% angles(:,1) = 360-angles(:,1);
%% process ITD
% smooth itds
delta = o * 48000;
sfft = fft(delta);
cutoff= 1000;
sfft(cutoff:end-cutoff+1) = 0;
delta_sm = real(ifft(sfft));
delta_sm_abs = max(abs(delta_sm));
%% get angles 
lebedev_order = 5810;

% extract angles
% angles = HRTF.SourcePosition;
[angles_cart(:,1),angles_cart(:,2),angles_cart(:,3)] = sph2cart(angles(:,1),angles(:,2),angles(:,3));

%
leb_grid_points = sofia_lebedev(lebedev_order); % [1] 
leb_grid_points_deg = rad2deg(leb_grid_points(:,1:2));
leb_grid_points_deg(:,2) = 90- leb_grid_points_deg(:,2);
[lbg_cart(:,1),lbg_cart(:,2),lbg_cart(:,3)] = sph2cart(leb_grid_points_deg(:,1), leb_grid_points_deg(:,2), ones(lebedev_order,1));

for k = 1:length(leb_grid_points_deg)
%     [custom_grid(k,:),index(k),di(k)] = findClosestPointOnSphere2(angles,[leb_grid_points_deg(k,:),1]);
    [custom_grid(k,:),index(k,1)] = findClosestPoint(leb_grid_points_deg(k,:),angles(:,1:2));

end

dt = permute(HRIR_MP,[2,3,1]);
dt = dt(:,:,index);
s_angles = angles(index,:);
s_delta_sm = delta_sm(index);
% itds = zeros(2,length(idx_unique));
itds = zeros(2,length(index));
hrir_iir_coeffs = zeros(2*(33+17), length(index));

%% generate filters
order_a = 16;
order_b = 32;

parfor n = 1:length(s_angles)

    [~,iir_l,iir_r] = generate_iir(squeeze(dt(:,:,n)), order_a, order_b);
    hrir_iir_coeffs(:,n) = [iir_l.Numerator' ; iir_l.Denominator'; iir_r.Numerator'; iir_r.Denominator'];
    
end

% gen delay from itds;
max_buffer_needed = ceil(delta_sm_abs);

for k = 1:length(index)
    itds(1,k) = max_buffer_needed/2 + s_delta_sm(k)/2;
    itds(2,k) = max_buffer_needed/2 - s_delta_sm(k)/2;
end

fid = fopen('hrir_iir_coeffs_2.dat','w');
fwrite(fid,hrir_iir_coeffs,'float32','b'); 
fclose(fid)

fid = fopen('hrir_iir_delays_2.dat','w');
fwrite(fid,itds,'float32','b'); 
fclose(fid)

s_angles(:,1) = 360-s_angles(:,1);
fid = fopen('hrir_iir_angles_2.dat','w');
fwrite(fid,(s_angles(:,1:2))','float32','b'); 
fclose(fid)




%% test [4640 4687]
% HRTF_RAW = permute(squeeze(HRTF_RAW(index,:,:)),[3,2,1]);
% HRIR_MP = permute(squeeze(HRIR_MP(index,:,:)),[3,2,1]);
if test 
    [sig, fs] = audioread('D:\Programming\Matlab\_Stimuli__speech__HarvardMale.wav');
    sig_st = [sig sig];
    
    id = [1324,1293];
    for k = 1:length(id)
    disp(angles(id,1:2));
    hrir_mp = squeeze(HRIR_MP(id(k),:,:));
%     hrir_orig = squeeze(HRTF_RAW(:,:,4640));
    [sig, fs] = audioread('D:\Programming\Matlab\_Stimuli__speech__HarvardMale.wav');
    sig_st = [sig sig];
    
    % iir 
%     order_b=32;
%     order_a= 16;
%     [hiir,iir_l,iir_r] = generate_iir(hrir_mp, order_a, order_b);
    iir = hrir_iir_coeffs(:,id(k)) ;
    % delayline
%     delay = delta_sm(id);
    vdl = dsp.VariableFractionalDelay("InterpolationMethod","FIR");
    sigst(:,1) = vdl(sig, itds(1,id(k)));
    sigst(:,2) = vdl(sig, itds(2,id(k)));
    
%     sound(fftfilt(hrir_orig, sig)*.5,fs);
%     sound([fftfilt(hiir(:,1), sigst(:,1)) fftfilt(hiir(:,2),sigst(:,2))]*.5,fs)
%     pause(5);
    sound([filter(iir(1:33),iir(34:50),sigst(:,1)) filter(iir(51:83),iir(84:100),sigst(:,2))]*.5,fs)
    pause(5);
    end
end 

%% helper functions
% function hiir = generate_iir(h, order_a, order_b)
% % i = 52652;
% % di = delta(i);
% % h = [zeros(abs(di),2); squeeze(HRIR_MP(i,:,:)); zeros(abs(di),2)];
% iir_l = dsp.IIRFilter;
% iir_r = dsp.IIRFilter;
% order=40;
% [bcl,acl] = stmcb(h(:,1),order_b,order_a);
% [bcr,acr] = stmcb(h(:,2),order,order_a);
% iir_l.Numerator = bcl;
% iir_l.Denominator = acl;
% iir_r.Numerator = bcr;
% iir_r.Denominator = acr;
% hiir(:,1) = iir_l.impz(256);
% hiir(:,2) = iir_r.impz(256);
% 
% end