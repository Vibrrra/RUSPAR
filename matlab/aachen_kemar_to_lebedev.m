%% Generating hrtf asset for the rendering
% Author: Christian Schneiderwind TU Ilmenau, (c) 2024.
%
% This script generates a subsampled version of the Aachen KEMAR HRTF dataset
% Download here: https://publications.rwth-aachen.de/record/807373
% This requires the SOFA Toolbbox: https://github.com/sofacoustics/SOFAtoolbox
% 
% A Lebedev-Grid is used for the downsampling. Following degrees of
% lebedev grids are availabler
% degrees_avail=[6, 14, 26, 38, 50, 74, 86, 110, 146, 170, 194, 230,... 
%   266, 302, 350, 434, 590, 770, 974, 1202, 1454, 1730, 2030, 2354, 2702,... 
%   3074, 3470, 3890, 4334, 4802, 5294, 5810];
%
% Angles are stored in pairs of azimuth and elevation.
% azimuth: [0 ... 360], elevation: [-90 .. 90]
%
% Float in the byte arrays are stored with the BigEndian tag. 

hrtf_bytes_name = "hrtf_binary.dat";
angle_bytes_name = "angles.dat";
float_endian_type = 'b';

leb_grid_degs = 2702; % see comments above or doc of function [1] for more details

% sub sample Aachen HRTF dataset;
if ~exist("HRTF","var")
    SOFAstart;
    HRTF = SOFAload("Kemar_HRTF_sofa.sofa");
end

% extract angles
angles = HRTF.SourcePosition;
[angles_cart(:,1),angles_cart(:,2),angles_cart(:,3)] = sph2cart(angles(:,1),angles(:,2),angles(:,3));

%
leb_grid_points = sofia_lebedev(leb_grid_degs); % [1] 
leb_grid_points_deg = rad2deg(leb_grid_points(:,1:2));
leb_grid_points_deg(:,2) = 90- leb_grid_points_deg(:,2);
[lbg_cart(:,1),lbg_cart(:,2),lbg_cart(:,3)] = sph2cart(leb_grid_points_deg(:,1), leb_grid_points_deg(:,2), ones(leb_grid_degs,1));
%leb_grid_points_deg(:,1) = mod(mod((leb_grid_points_deg(:,1)+179),360) +360 ,360) -179;
% find closest point

% find the closest hrtf pairs 
for k = 1:length(leb_grid_points_deg)
    [custom_grid(k,:),index(k)] = findClosestPointOnSphere2(angles,[leb_grid_points_deg(k,:),1]);
end

% discard duplicate hrtfs 
idx_unique = unique(index);
dt = permute(HRTF.Data.IR,[3,2,1]);
dt = dt(:,:,idx_unique);
ang = HRTF.SourcePosition(idx_unique,:);

% reshape // found-out via trial and error 
hrtfs = reshape(dt,384,2*2588);
angs = ang(:,1:2)';

%% Write byte arrays, 
fid = fopen('hrtf_binary.dat','w');
fwrite(fid,hrtfs,'float32','b');
fclose(fid);
fid = fopen('angles.dat','w');
fwrite(fid,angs,'float32','b');
fclose(fid);
